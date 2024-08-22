// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::apply::{RaftMessage, RaftResponseMesage};
use super::route::DataRoute;
use super::storage::RaftRocksDBStorage;
use crate::raft::metadata::RaftGroupMetadata;
use crate::raft::peer::PeerMessage;
use crate::storage::placement::raft::RaftMachineStorage;
use bincode::{deserialize, serialize};
use common_base::config::placement_center::placement_center_conf;
use log::{error, info};
use metadata_struct::placement::broker_node::BrokerNode;
use prost::Message as _;
use raft::eraftpb::{
    ConfChange, ConfChangeType, Entry, EntryType, Message as raftPreludeMessage, MessageType,
    Snapshot,
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
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::timeout;

pub struct RaftMachine {
    placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
    receiver: Receiver<RaftMessage>,
    seqnum: AtomicUsize,
    resp_channel: HashMap<usize, oneshot::Sender<RaftResponseMesage>>,
    data_route: Arc<RwLock<DataRoute>>,
    entry_num: AtomicUsize,
    peer_message_send: Sender<PeerMessage>,
    stop_recv: broadcast::Receiver<bool>,
    raft_storage: Arc<RwLock<RaftMachineStorage>>,
}

impl RaftMachine {
    pub fn new(
        placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
        data_route: Arc<RwLock<DataRoute>>,
        peer_message_send: Sender<PeerMessage>,
        receiver: Receiver<RaftMessage>,
        stop_recv: broadcast::Receiver<bool>,
        raft_storage: Arc<RwLock<RaftMachineStorage>>,
    ) -> Self {
        let seqnum = AtomicUsize::new(1);
        let entry_num = AtomicUsize::new(1);
        let resp_channel = HashMap::new();
        return Self {
            placement_cluster,
            receiver,
            seqnum,
            resp_channel,
            data_route,
            entry_num,
            peer_message_send,
            stop_recv,
            raft_storage,
        };
    }

    pub async fn run(&mut self) {
        let mut raft_node: RawNode<RaftRocksDBStorage> = self.new_node().await;
        let heartbeat = Duration::from_millis(100);
        let mut now = Instant::now();
        loop {
            match self.stop_recv.try_recv() {
                Ok(val) => {
                    if val {
                        info!("{}", "Raft Node Process services stop.");
                        break;
                    }
                }
                Err(_) => {}
            }

            match timeout(heartbeat, self.receiver.recv()).await {
                Ok(Some(RaftMessage::ConfChange { change, chan })) => {
                    let seq = self
                        .seqnum
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    match raft_node.propose_conf_change(serialize(&seq).unwrap(), change) {
                        Ok(_) => {
                            self.resp_channel.insert(seq, chan);
                        }
                        Err(e) => {
                            error!("{}", e,);
                        }
                    }
                }

                Ok(Some(RaftMessage::Raft { message, chan })) => {
                    // Step advances the state machine using the given message.

                    match raft_node.step(message) {
                        // After the step message succeeds, you can return success directly
                        Ok(_) => match chan.send(RaftResponseMesage::Success) {
                            Ok(_) => {}
                            Err(_) => {
                                error!("{}","commit entry Fails to return data to chan. chan may have been closed");
                            }
                        },
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }

                Ok(Some(RaftMessage::TransferLeader { node_id, chan })) => {
                    // Step advances the state machine using the given message.
                    info!("transfer_leader {}", node_id);
                    raft_node.transfer_leader(node_id);
                    match chan.send(RaftResponseMesage::Success) {
                        Ok(_) => {}
                        Err(_) => {
                            error!("{}","commit entry Fails to return data to chan. chan may have been closed");
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
                            self.resp_channel.insert(seq, chan);
                        }
                        Err(e) => {
                            error!("{}", e);
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

            if self.placement_cluster.read().unwrap().raft_role != raft_node.raft.state {
                info!(
                    "Node Raft Role changes from  【{:?}】 to 【{:?}】",
                    self.placement_cluster.read().unwrap().raft_role,
                    raft_node.raft.state
                );
                self.placement_cluster
                    .write()
                    .unwrap()
                    .set_role(raft_node.raft.state)
            }
            // info!(&format!("{:?}",raft_node.raft.state));
            self.on_ready(&mut raft_node).await;
        }
    }

    async fn on_ready(&mut self, raft_node: &mut RawNode<RaftRocksDBStorage>) {
        if !raft_node.has_ready() {
            return;
        }

        let mut ready = raft_node.ready();
        // After receiving the data sent by the client,
        // the data needs to be sent to other Raft nodes for persistent storage.
        if !ready.messages().is_empty() {
            self.send_message(ready.take_messages()).await;
        }

        // If the snapshot is not empty, save the snapshot to Storage, and apply
        // the data in the snapshot to the State Machine asynchronously.
        // (Although synchronous apply can also be applied here,
        // but the snapshot is usually large. Synchronization blocks threads).
        if *ready.snapshot() != Snapshot::default() {
            let s = ready.snapshot().clone();
            info!(
                "save snapshot,term:{},index:{}",
                s.get_metadata().get_term(),
                s.get_metadata().get_index()
            );
            raft_node.mut_store().apply_snapshot(s).unwrap();
        }

        // messages need to be stored to Storage before they can be sent.Save entries to Storage.
        if !ready.entries().is_empty() {
            let entries = ready.entries();
            raft_node.mut_store().append(entries).unwrap();
        }

        // The committed raft log can be applied to the State Machine.
        self.handle_committed_entries(raft_node, ready.take_committed_entries());

        // If there is a change in HardState, such as a revote,
        // term is increased, the hs will not be empty.Persist non-empty hs.
        if let Some(hs) = ready.hs() {
            info!("save hardState!!!,len:{:?}", hs);
            raft_node.mut_store().set_hard_state(hs.clone()).unwrap();
        }

        // Persisted Messages specifies outbound messages to be sent AFTER the HardState,
        // Entries and Snapshot are persisted to stable storage.
        if !ready.persisted_messages().is_empty() {
            self.send_message(ready.take_persisted_messages()).await;
        }

        // A call to advance tells Raft that it is ready for processing.
        let mut light_rd = raft_node.advance(ready);
        if let Some(commit) = light_rd.commit_index() {
            info!("save light rd!!!,commit:{:?}", commit);
            raft_node.mut_store().set_hard_state_comit(commit).unwrap();
        }

        self.send_message(light_rd.take_messages()).await;

        self.handle_committed_entries(raft_node, light_rd.take_committed_entries());

        raft_node.advance_apply();
    }

    fn handle_committed_entries(
        &mut self,
        raft_node: &mut RawNode<RaftRocksDBStorage>,
        entrys: Vec<Entry>,
    ) {
        let data_route = self.data_route.write().unwrap();
        for entry in entrys {
            if !entry.data.is_empty() {
                info!("ready entrys entry type:{:?}", entry.get_entry_type());
                match entry.get_entry_type() {
                    EntryType::EntryNormal => {
                        // Saves the service data sent by the client
                        match data_route.route(entry.get_data().to_vec()) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("{}", err);
                            }
                        }
                    }
                    EntryType::EntryConfChange => {
                        let change = ConfChange::decode(entry.get_data())
                            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                            .unwrap();
                        let id = change.get_node_id();
                        let change_type = change.get_change_type();
                        match change_type {
                            ConfChangeType::AddNode => {
                                match deserialize::<BrokerNode>(change.get_context()) {
                                    Ok(node) => {
                                        let mut cls = self.placement_cluster.write().unwrap();
                                        cls.add_peer(id, node);
                                    }
                                    Err(e) => {
                                        error!("Failed to parse Node data from context with error message {:?}", e);
                                    }
                                }
                            }
                            ConfChangeType::RemoveNode => {
                                let mut cls = self.placement_cluster.write().unwrap();
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
            }

            let idx: u64 = entry.get_index();
            let _ = raft_node.mut_store().commmit_index(idx);

            match deserialize(entry.get_context()) {
                Ok(seq) => match self.resp_channel.remove(&seq) {
                    Some(chan) => match chan.send(RaftResponseMesage::Success) {
                        Ok(_) => {}
                        Err(_) => {
                            error!("commit entry Fails to return data to chan. chan may have been closed");
                        }
                    },
                    None => {}
                },
                Err(_) => {}
            }

            self.create_snapshot(raft_node);
        }
    }

    async fn send_message(&self, messages: Vec<raftPreludeMessage>) {
        for msg in messages {
            let to = msg.get_to();
            if msg.get_msg_type() != MessageType::MsgHeartbeat
                && msg.get_msg_type() != MessageType::MsgHeartbeatResponse
            {
                info!("ready message:{:?}", msg);
            }
            let data: Vec<u8> = raftPreludeMessage::encode_to_vec(&msg);
            self.send_peer_message(to, data).await;
        }
    }

    pub async fn new_node(&self) -> RawNode<RaftRocksDBStorage> {
        let cluster = self.placement_cluster.read().unwrap();
        let storage = RaftRocksDBStorage::new(self.raft_storage.clone());

        // build config
        let hs = storage.read_lock().hard_state();
        let conf = self.build_config(hs.commit);

        // init voters && learns
        let mut cs = storage.read_lock().conf_state();
        cs.voters = cluster.node_ids();
        let _ = storage.write_lock().save_conf_state(cs);

        let logger = self.build_slog();
        let node = RawNode::new(&conf, storage, &logger).unwrap();
        return node;
    }

    fn build_config(&self, apply: u64) -> Config {
        let conf = placement_center_conf();
        Config {
            // The unique ID for the Raft node.
            // id: self.config.node_id,
            id: conf.node_id,
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
            applied: apply,
            // check_quorum: true,
            ..Default::default()
        }
    }

    fn build_slog(&self) -> slog::Logger {
        let conf = placement_center_conf();
        let path = format!("{}/raft.log", conf.log.log_path.clone());
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

    pub async fn send_peer_message(&self, id: u64, msg: Vec<u8>) {
        if let Some(node) = self.placement_cluster.read().unwrap().get_node_by_id(id) {
            let send = self.peer_message_send.clone();
            let node_c = node.clone();
            tokio::spawn(async move {
                match send
                    .send(PeerMessage {
                        to: node_c.node_inner_addr,
                        data: msg,
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(e) => error!(
                        "Failed to write Raft Message to send queue with error message: {:?}",
                        e.to_string()
                    ),
                }
            });
        } else {
            error!("raft message was sent to node {}, but the node information could not be found. It may be that the node is not online yet.",id);
        }
    }
}
