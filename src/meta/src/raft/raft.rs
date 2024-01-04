use super::election::Election;
use super::message::{RaftMessage, RaftResponseMesage};
use crate::client::send_raft_conf_change;
use crate::cluster::Cluster;
use crate::errors::MetaError;
use crate::raft::peer::Peer;
use crate::storage::raft_storage::RaftRocksDBStorage;
use crate::storage::route::DataRoute;
use crate::Node;
use bincode::{deserialize, serialize};
use common::config::meta::MetaConfig;
use common::log::{error_meta, info_meta};
use prost::Message as _;
use raft::eraftpb::{
    ConfChange, ConfChangeType, Entry, EntryType, Message as raftPreludeMessage, Snapshot,
};
use raft::{Config, RawNode};
use slog::o;
use slog::Drain;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;
use tokio::time::timeout;

pub struct MetaRaft {
    config: MetaConfig,
    cluster: Arc<RwLock<Cluster>>,
    receiver: Receiver<RaftMessage>,
    seqnum: AtomicUsize,
    resp_channel: HashMap<usize, oneshot::Sender<RaftResponseMesage>>,
    storage: Arc<RwLock<DataRoute>>,
    entry_num: AtomicUsize,
}

impl MetaRaft {
    pub fn new(
        config: MetaConfig,
        cluster: Arc<RwLock<Cluster>>,
        storage: Arc<RwLock<DataRoute>>,
        receiver: Receiver<RaftMessage>,
    ) -> Self {
        let seqnum = AtomicUsize::new(1);
        let entry_num = AtomicUsize::new(1);
        let resp_channel = HashMap::new();
        return Self {
            config,
            cluster,
            receiver,
            seqnum,
            resp_channel,
            storage,
            entry_num,
        };
    }

    pub async fn ready(&mut self) {
        info_meta("Meta raft state machine is being initialized");
        let leader_node = self.get_leader_node().await;
        info_meta(&format!("Meta cluster Leader ID:{}", leader_node.id));
        self.run(leader_node).await;
    }

    async fn get_leader_node(&self) -> Node {
        let mata_nodes = self.config.meta_nodes.clone();
        if mata_nodes.len() == 1 {
            return Node::new(
                self.config.addr.clone(),
                self.config.node_id.clone(),
                self.config.port.clone(),
            );
        }

        // Leader Election
        let elec = Election::new(mata_nodes);
        let ld = match elec.leader_election().await {
            Ok(nd) => nd,
            Err(err) => {
                error_meta(&format!(
                    "When a node fails to obtain the Leader from another node during startup, 
                the current node is set to the Leader node. Error message {}",
                    err
                ));

                // todo We need to deal with the split-brain problem here. We'll deal with it later
                return Node::new(
                    self.config.addr.clone(),
                    self.config.node_id.clone(),
                    self.config.port.clone(),
                );
            }
        };

        info_meta(&format!("cluster Leader is {}", ld));
        return ld;
    }

    pub async fn run(&mut self, leader_node: Node) {
        let mut raft_node = if self.config.node_id == leader_node.id {
            self.new_leader()
        } else {
            self.new_follower(leader_node).await
        };

        let heartbeat = Duration::from_millis(100);
        let mut now = Instant::now();
        loop {
            match timeout(heartbeat, self.receiver.recv()).await {
                Ok(Some(RaftMessage::ConfChange { change, chan })) => {
                    let seq = self
                        .seqnum
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    match raft_node.propose_conf_change(serialize(&seq).unwrap(), change) {
                        Ok(_) => {
                            self.resp_channel.insert(seq, chan).unwrap();
                        }
                        Err(e) => {
                            error_meta(
                                &MetaError::RaftConfChangeCommitFail(e.to_string()).to_string(),
                            );
                        }
                    }
                }

                Ok(Some(RaftMessage::Raft { message, chan })) => {
                    // Step advances the state machine using the given message.
                    let seq = self
                        .seqnum
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    match raft_node.step(message) {
                        Ok(_) => {
                            self.resp_channel.insert(seq, chan).unwrap();
                        }
                        Err(e) => {
                            error_meta(&MetaError::RaftStepCommitFail(e.to_string()).to_string());
                        }
                    }
                }

                Ok(Some(RaftMessage::Propose { data, chan })) => {
                    // Propose proposes data be appended to the raft log.
                    let seq = self
                        .seqnum
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    match raft_node.propose(serialize(&seq).unwrap(), data) {
                        Ok(_) => {
                            self.resp_channel.insert(seq, chan).unwrap();
                        }
                        Err(e) => {
                            error_meta(
                                &MetaError::RaftProposeCommitFail(e.to_string()).to_string(),
                            );
                        }
                    }
                }
                Ok(None) => continue,
                Err(_) => {}
            }

            let elapsed = now.elapsed();

            if elapsed >= heartbeat {
                raft_node.tick();
                now = Instant::now();
            }

            self.on_ready(&mut raft_node).await;
        }
    }

    async fn on_ready(&mut self, raft_node: &mut RawNode<RaftRocksDBStorage>) {
        if !raft_node.has_ready() {
            return;
        }

        let mut ready = raft_node.ready();

        info_meta("raft on ready");
        // After receiving the data sent by the client,
        // the data needs to be sent to other Raft nodes for persistent storage.
        if !ready.messages().is_empty() {
            info_meta(&format!("save message!!!,len:{}", ready.messages().len()));
            self.send_message(ready.take_messages());
        }

        // If the snapshot is not empty, save the snapshot to Storage, and apply
        // the data in the snapshot to the State Machine asynchronously.
        // (Although synchronous apply can also be applied here,
        // but the snapshot is usually large. Synchronization blocks threads).
        if *ready.snapshot() != Snapshot::default() {
            let s = ready.snapshot().clone();
            info_meta(&format!(
                "save snapshot,term:{},index:{}",
                s.get_metadata().get_term(),
                s.get_metadata().get_index()
            ));
            raft_node.mut_store().apply_snapshot(s).unwrap();
        }

        // messages need to be stored to Storage before they can be sent.Save entries to Storage.
        if !ready.entries().is_empty() {
            info_meta(&format!("save entries!!!,len:{}", ready.entries().len()));
            let entries = ready.entries();
            raft_node.mut_store().append(entries).unwrap();
        }

        // The committed raft log can be applied to the State Machine.
        self.handle_committed_entries(raft_node, ready.take_committed_entries());

        // If there is a change in HardState, such as a revote,
        // term is increased, the hs will not be empty.Persist non-empty hs.
        if let Some(hs) = ready.hs() {
            info_meta(&format!("save hardState!!!,len:{:?}", hs));
            raft_node.mut_store().set_hard_state(hs.clone()).unwrap();
        }

        // Persisted Messages specifies outbound messages to be sent AFTER the HardState,
        // Entries and Snapshot are persisted to stable storage.
        if !ready.persisted_messages().is_empty() {
            info_meta(&format!(
                "save persisted_messages!!!,len:{:?}",
                ready.persisted_messages().len()
            ));
            self.send_message(ready.take_persisted_messages());
        }

        // A call to advance tells Raft that it is ready for processing.
        let mut light_rd = raft_node.advance(ready);
        if let Some(commit) = light_rd.commit_index() {
            info_meta(&format!("save light rd!!!,commit:{:?}", commit));
            raft_node.mut_store().set_hard_state_comit(commit).unwrap();
        }

        self.send_message(light_rd.take_messages());

        self.handle_committed_entries(raft_node, light_rd.take_committed_entries());

        raft_node.advance_apply();
    }

    fn handle_committed_entries(
        &mut self,
        raft_node: &mut RawNode<RaftRocksDBStorage>,
        entrys: Vec<Entry>,
    ) {
        info_meta(&format!(
            "handle committed entries !!!,len:{}",
            entrys.len()
        ));

        let storage = self.storage.write().unwrap();
        for entry in entrys {
            println!("{:?}", entry.get_entry_type());
            println!("{:?}", entry.get_data());
            if entry.data.is_empty() {
                continue;
            }

            match entry.get_entry_type() {
                EntryType::EntryNormal => {
                    let idx: u64 = entry.get_index();
                    let _ = raft_node.mut_store().commmit_index(idx);

                    // Saves the service data sent by the client
                    match storage.route(entry.get_data().to_vec()) {
                        Ok(_) => {}
                        Err(err) => {
                            error_meta(&err.to_string());
                        }
                    }
                }
                EntryType::EntryConfChange => {
                    let change = ConfChange::decode(entry.get_data())
                        .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                        .unwrap();
                    let id = change.get_node_id();
                    let addr: String = deserialize(change.get_context()).unwrap();
                    let change_type = change.get_change_type();

                    match change_type {
                        ConfChangeType::AddNode => {
                            let mut cls = self.cluster.write().unwrap();
                            // let addr = cls.get_addr_by_id(id);
                            let peer = Peer::new(addr);
                            cls.add_peer(id, peer)
                        }
                        ConfChangeType::RemoveNode => {
                            let mut cls = self.cluster.write().unwrap();
                            cls.remove_peer(id);
                        }
                        _ => unimplemented!(),
                    }

                    if let Ok(cs) = raft_node.apply_conf_change(&change) {
                        let _ = raft_node.mut_store().set_conf_state(cs);
                    }
                }
                EntryType::EntryConfChangeV2 => {}
            }

            let seq: usize = deserialize(entry.get_context()).unwrap();
            match self.resp_channel.remove(&seq) {
                Some(chan) => {
                    match chan.send(RaftResponseMesage::Success) {
                        Ok(_) => {}
                        Err(_) => {
                            error_meta("commit entry Fails to return data to chan. chan may have been closed");
                        }
                    }
                }
                None => {}
            }

            self.create_snapshot(raft_node);
        }
    }

    fn send_message(&self, messages: Vec<raftPreludeMessage>) {
        for msg in messages {
            let to = msg.get_to();
            let data: Vec<u8> = raftPreludeMessage::encode_to_vec(&msg);
            let mut cluster = self.cluster.write().unwrap();
            cluster.send_message(to, data);
        }
    }

    fn new_leader(&self) -> RawNode<RaftRocksDBStorage> {
        let conf = self.build_config();
        let storage = RaftRocksDBStorage::new(&self.config);
        let logger = self.build_slog();
        let mut node = RawNode::new(&conf, storage, &logger).unwrap();

        // Change the role of the current node to Leader
        node.raft.become_candidate();
        node.raft.become_leader();
        return node;
    }

    pub async fn new_follower(&self, leader_node: Node) -> RawNode<RaftRocksDBStorage> {
        let conf = self.build_config();
        let storage = RaftRocksDBStorage::new(&self.config);
        let logger = self.build_slog();
        let node = RawNode::new(&conf, storage, &logger).unwrap();

        // Add the leader node to the peer list
        let mut cluster = self.cluster.write().unwrap();
        cluster.add_peer(leader_node.id, Peer::new(leader_node.addr()));

        // try remove from the cluster
        let mut change = ConfChange::default();
        change.set_node_id(self.config.node_id);
        change.set_change_type(ConfChangeType::RemoveNode);
        match send_raft_conf_change(&leader_node.addr(), ConfChange::encode_to_vec(&change)).await {
            Ok(_) => {}
            Err(err) => {
                info_meta(&format!(
                    "Attempts to remove the current node from the 
                cluster after initializing a Follower node fail with the error message {}",
                    err.to_string()
                ));
            }
        }

        // Join the cluster
        let mut change = ConfChange::default();
        change.set_node_id(self.config.node_id);
        change.set_change_type(ConfChangeType::AddNode);
        change.set_context(
            serialize(&format!(
                "{}:{}",
                self.config.addr.clone(),
                self.config.port.clone()
            ))
            .unwrap(),
        );
        match send_raft_conf_change(&leader_node.addr(), ConfChange::encode_to_vec(&change)).await {
            Ok(_) => {}
            Err(err) => {
                info_meta(&format!(
                    "Error occurs when initializing a Follower node and adding the 
                node to the cluster with the error message {}",
                    err.to_string()
                ));
            }
        }
        return node;
    }

    fn build_config(&self) -> Config {
        Config {
            // The unique ID for the Raft node.
            id: 1,
            // Election tick is for how long the follower may campaign again after
            // it doesn't receive any message from the leader.
            election_tick: 10,
            // Heartbeat tick is for how long the leader needs to send
            // a heartbeat to keep alive.
            heartbeat_tick: 3,
            // The max size limits the max size of each appended message. Mostly, 1 MB is enough.
            max_size_per_msg: 1024 * 1024 * 1024,
            // Max inflight msgs that the leader sends messages to follower without
            // receiving ACKs.
            max_inflight_msgs: 256,
            // The Raft applied index.
            // You need to save your applied index when you apply the committed Raft logs.
            applied: 0,
            ..Default::default()
        }
    }

    fn build_slog(&self) -> slog::Logger {
        let path = format!("{}/raft.log", self.config.log_path.clone());
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)
            .unwrap();

        let decorator = slog_term::PlainDecorator::new(file);
        // let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain)
            .chan_size(4096)
            .overflow_strategy(slog_async::OverflowStrategy::Block)
            .build()
            .fuse();
        let logger = slog::Logger::root(drain, o!("tag" => format!("meta-node-id={}", 1)));
        return logger;
    }

    fn create_snapshot(&self, raft_node: &mut RawNode<RaftRocksDBStorage>) {
        let num = self
            .entry_num
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if num % 1000 == 0 {
            raft_node.mut_store().create_snapshot().unwrap();
        }
    }
}
