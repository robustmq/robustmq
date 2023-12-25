#![allow(clippy::field_reassign_with_default)]
use super::election::Election;
use super::message::RaftMessage;
use super::node::Node;
use crate::storage::raft_storage::RaftRocksDBStorage;
use common::config::meta::MetaConfig;
use common::log::{error_meta, info, info_meta};

use raft::{Config, RawNode};
use raft_proto::eraftpb::{ConfChangeType, Entry, EntryType, ConfChange};
use raft_proto::eraftpb::{ConfChangeV2, HardState, Snapshot};
use raft_proto::prelude::Message as raftPreludeMessage;
use protobuf::Message as PbMessage;

use raft_proto::parse_conf_change;
use slog::o;
use slog::Drain;
use std::fs::OpenOptions;
use std::str::from_utf8;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;

pub struct MetaRaft {
    config: MetaConfig,
    receiver: Receiver<RaftMessage>,
}

impl MetaRaft {
    pub fn new(config: MetaConfig, receiver: Receiver<RaftMessage>) -> Self {
        return Self {
            config: config,
            receiver: receiver,
        };
    }

    pub async fn ready(&mut self) {
        let leader_node = self.get_leader_node().await;
        info(&format!(
            "The leader address of the cluster is {} and the node ID is {}",
            leader_node.node_ip, leader_node.node_id
        ));
        self.run(leader_node).await;
    }

    async fn get_leader_node(&self) -> Node {
        let mata_nodes = self.config.meta_nodes.clone();
        if mata_nodes.len() == 1 {
            return Node::new(self.config.addr.clone(), self.config.node_id.clone());
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
                return Node::new(self.config.addr.clone(), self.config.node_id.clone());
            }
        };

        info_meta(&format!("cluster Leader is {}", ld));
        return ld;
    }

    pub async fn run(&mut self, leader_node: Node) {
        let mut raft_node = if self.config.node_id == leader_node.node_id {
            self.new_leader()
        } else {
            self.new_follower()
        };

        let heartbeat = Duration::from_millis(100);
        let mut now = Instant::now();
        loop {
            match timeout(heartbeat, self.receiver.recv()).await {
                Ok(Some(RaftMessage::Raft(msg))) => {
                    // Step advances the state machine using the given message.
                    let _ = raft_node.step(msg);
                }
                Ok(Some(RaftMessage::Propose { data, chan })) => {
                    // Propose proposes data be appended to the raft log.
                    let _ = raft_node.propose(vec![], data);
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

    async fn on_ready(&self, raft_node: &mut RawNode<RaftRocksDBStorage>) {
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

        // The committed raft log can be applied to the State Machine.
        self.handle_committed_entries(raft_node, ready.take_committed_entries());

        // messages need to be stored to Storage before they can be sent.Save entries to Storage.
        if !ready.entries().is_empty() {
            info_meta(&format!("save entries!!!,len:{}", ready.entries().len()));
            let entries = ready.entries();
            raft_node.mut_store().append(entries).unwrap();
        }

        // If there is a change in HardState, such as a revote,
        // term is increased, the hs will not be empty.Persist non-empty hs.
        if let Some(hs) = ready.hs() {
            info_meta(&format!("save hardState!!!,len:{:?}", hs));
            let mut new_hs = HardState::default();
            new_hs.set_commit(hs.get_commit());
            new_hs.set_term(hs.get_term());
            new_hs.set_vote(hs.get_vote());
            raft_node.mut_store().set_hard_state(new_hs).unwrap();
        }

        //
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
        &self,
        raft_node: &mut RawNode<RaftRocksDBStorage>,
        entrys: Vec<Entry>,
    ) {
        info_meta(&format!(
            "handle committed entries !!!,len:{}",
            entrys.len()
        ));
        for entry in entrys {
            if entry.data.is_empty() {
                continue;
            }

            match entry.get_entry_type() {
                EntryType::EntryNormal => {
                    // For normal proposals, extract the key-value pair and then
                    // insert them into the kv engine.
                    let idx: u64 = entry.get_index();
                    let _ = raft_node.mut_store().commmit_index(idx);
                }
                EntryType::EntryConfChange => {
                    // todo 

                    // For conf change messages, make them effective.
                    // prostMessage::decode(buf).unwrap();
                    let str = from_utf8(entry.get_data()).unwrap();
                    let change_list = parse_conf_change(str).unwrap();
                    for cfs in change_list {
                        let change_type = cfs.get_change_type();
                        let node_id = cfs.get_node_id();
                        match change_type {
                            ConfChangeType::AddNode => {
                                // save
                            }
                            ConfChangeType::RemoveNode => {}
                            ConfChangeType::AddLearnerNode => {}
                        }

                        // if let Ok(cs) = raft_node.apply_conf_change(&cfs){

                        // }
                    }
                    let mut cc = ConfChange::default();
                    // raft_node.propose_conf_change(context, cc)
                }
                EntryType::EntryConfChangeV2 => {}
            }
        }
    }

    fn send_message(&self, messages: Vec<raftPreludeMessage>) {
        for msg in messages {
            // println!("{:?}", print_to_string(&msg));
        }
    }

    fn new_leader(&self) -> RawNode<RaftRocksDBStorage> {
        let conf = self.build_config();
        let mut s = Snapshot::default();

        // Because we don't use the same configuration to initialize every node, so we use
        // a non-zero index to force new followers catch up logs by snapshot first, which will
        // bring all nodes to the same initial state.
        s.mut_metadata().index = 1;
        s.mut_metadata().term = 1;
        s.mut_metadata().mut_conf_state().voters = vec![self.config.node_id];

        let mut storage = RaftRocksDBStorage::new(&self.config);
        let _ = storage.apply_snapshot(s);

        let logger = self.build_slog();
        let mut node = RawNode::new(&conf, storage, &logger).unwrap();
        node.raft.become_candidate();
        node.raft.become_leader();
        return node;
    }

    pub fn new_follower(&self) -> RawNode<RaftRocksDBStorage> {
        let conf = self.build_config();
        let storage = RaftRocksDBStorage::new(&self.config);
        let logger = self.build_slog();
        RawNode::new(&conf, storage, &logger).unwrap()
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
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(format!("./log/raft.log"))
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
}
