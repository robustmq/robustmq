// Copyright 2023 RobustMQ Team
//
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

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use bincode::{deserialize, serialize};
use common_base::config::placement_center::placement_center_conf;
use common_base::error::common::CommonError;
use log::{debug, error, info};
use prost::Message as _;
use raft::eraftpb::{
    ConfChange, ConfChangeType, Entry, EntryType, Message as raftPreludeMessage, Snapshot,
};
use raft::{Config, RawNode};
use slog::{o, Drain};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::timeout;

use super::rocksdb::RaftMachineStorage;
use super::storage::RaftRocksDBStorage;
use crate::cache::placement::PlacementCacheManager;
use crate::core::raft_node::RaftNode;
use crate::raftv1::peer::PeerMessage;
use crate::storage::route::apply::{RaftMessage, RaftResponseMesage};
use crate::storage::route::DataRoute;

pub struct RaftMachine {
    cache_placement: Arc<PlacementCacheManager>,
    receiver: Receiver<RaftMessage>,
    seqnum: AtomicUsize,
    resp_channel: HashMap<usize, oneshot::Sender<RaftResponseMesage>>,
    data_route: Arc<DataRoute>,
    peer_message_send: Sender<PeerMessage>,
    stop_recv: broadcast::Receiver<bool>,
    raft_storage: Arc<RwLock<RaftMachineStorage>>,
    local_node_id: u64,
}

impl RaftMachine {
    pub fn new(
        cache_placement: Arc<PlacementCacheManager>,
        data_route: Arc<DataRoute>,
        peer_message_send: Sender<PeerMessage>,
        receiver: Receiver<RaftMessage>,
        stop_recv: broadcast::Receiver<bool>,
        raft_storage: Arc<RwLock<RaftMachineStorage>>,
    ) -> Self {
        let seqnum = AtomicUsize::new(1);
        let resp_channel = HashMap::new();
        let conf = placement_center_conf();
        Self {
            cache_placement,
            receiver,
            seqnum,
            resp_channel,
            data_route,
            peer_message_send,
            stop_recv,
            raft_storage,
            local_node_id: conf.node.node_id,
        }
    }

    pub async fn run(&mut self) -> Result<(), CommonError> {
        let mut raft_node = match self.new_node() {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };

        let heartbeat = Duration::from_millis(100);
        let mut now = Instant::now();
        loop {
            if let Ok(val) = self.stop_recv.try_recv() {
                if val {
                    info!("{}", "Raft Node Process services stop.");
                    break;
                }
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
                            error!("raft node propose conf change fail, {}", e,);
                        }
                    }
                }

                Ok(Some(RaftMessage::Raft { message, chan })) => match raft_node.step(*message) {
                    Ok(_) => match chan.send(RaftResponseMesage::Success) {
                        Ok(_) => {}
                        Err(_) => {
                            error!("{}","commit entry Fails to return data to chan. chan may have been closed");
                        }
                    },
                    Err(e) => {
                        error!("raft node step fail,{}", e);
                    }
                },

                Ok(Some(RaftMessage::TransferLeader { node_id, chan })) => {
                    raft_node.transfer_leader(node_id);
                    match chan.send(RaftResponseMesage::Success) {
                        Ok(_) => {}
                        Err(_) => {
                            error!("{}","commit entry Fails to return data to chan. chan may have been closed");
                        }
                    }
                }

                Ok(Some(RaftMessage::Propose { data, chan })) => {
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

            if now.elapsed() >= heartbeat {
                raft_node.tick();
                now = Instant::now();
            }

            self.try_record_role_change(&raft_node);

            match self.on_ready(&mut raft_node).await {
                Ok(()) => {}
                Err(e) => {
                    error!("on ready error: {}", e.to_string())
                }
            }
        }
        Ok(())
    }

    async fn on_ready(
        &mut self,
        raft_node: &mut RawNode<RaftRocksDBStorage>,
    ) -> Result<(), CommonError> {
        if !raft_node.has_ready() {
            return Ok(());
        }

        let mut ready = raft_node.ready();
        // After receiving the data sent by the client,
        // the data needs to be sent to other Raft nodes for persistent storage.
        if !ready.messages().is_empty() {
            self.send_message(ready.take_messages()).await?;
        }

        // If the snapshot is not empty, save the snapshot to Storage, and apply
        // the data in the snapshot to the State Machine asynchronously.
        // (Although synchronous apply can also be applied here,
        // but the snapshot is usually large. Synchronization blocks threads).
        if *ready.snapshot() != Snapshot::default() {
            let s = ready.snapshot().clone();
            raft_node.mut_store().recovery_snapshot(s)?;
        }

        // messages need to be stored to Storage before they can be sent.Save entries to Storage.
        if !ready.entries().is_empty() {
            let entries = ready.entries();
            raft_node.mut_store().append_entries(entries.as_slice())?;
        }

        // The committed raft log can be applied to the State Machine.
        self.handle_committed_entries(raft_node, ready.take_committed_entries())?;

        // If there is a change in HardState, such as a revote,
        // term is increased, the hs will not be empty.Persist non-empty hs.
        if let Some(hs) = ready.hs() {
            debug!("save hardState!!!,len:{:?}", hs);
            raft_node.mut_store().set_hard_state(hs.clone())?;
        }

        // Persisted Messages specifies outbound messages to be sent AFTER the HardState,
        // Entries and Snapshot are persisted to stable storage.
        if !ready.persisted_messages().is_empty() {
            self.send_message(ready.take_persisted_messages()).await?;
        }

        // A call to advance tells Raft that it is ready for processing.
        let mut light_rd = raft_node.advance(ready);
        if let Some(commit) = light_rd.commit_index() {
            debug!("save light rd!!!,commit:{:?}", commit);
            raft_node.mut_store().set_hard_state_comit(commit)?;
        }

        self.send_message(light_rd.take_messages()).await?;

        self.handle_committed_entries(raft_node, light_rd.take_committed_entries())?;

        raft_node.advance_apply();
        Ok(())
    }

    fn handle_committed_entries(
        &mut self,
        raft_node: &mut RawNode<RaftRocksDBStorage>,
        entrys: Vec<Entry>,
    ) -> Result<(), CommonError> {
        for entry in entrys {
            if !entry.data.is_empty() {
                debug!("ready entrys entry type:{:?}", entry.get_entry_type());
                match entry.get_entry_type() {
                    EntryType::EntryNormal => {
                        // Saves the service data sent by the client
                        self.data_route.route_vec(entry.get_data().to_vec())?
                    }
                    EntryType::EntryConfChange => {
                        let change = ConfChange::decode(entry.get_data())?;
                        let id = change.get_node_id();
                        let change_type = change.get_change_type();
                        match change_type {
                            ConfChangeType::AddNode => {
                                let node = deserialize::<RaftNode>(change.get_context())?;
                                self.cache_placement.add_raft_memner(node);
                            }
                            ConfChangeType::RemoveNode => {
                                self.cache_placement.remove_raft_memner(id);
                            }
                            ConfChangeType::AddLearnerNode => {
                                //todo
                            }
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
            if !entry.get_context().is_empty() {
                let seq = deserialize(entry.get_context())?;
                if let Some(chan) = self.resp_channel.remove(&seq) {
                    match chan.send(RaftResponseMesage::Success) {
                        Ok(_) => {}
                        Err(_) => {
                            return Err(CommonError::CommmonError(
                                "commit entry Fails to return data to chan. chan may have been closed"
                                    .to_string(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn send_message(&self, messages: Vec<raftPreludeMessage>) -> Result<(), CommonError> {
        for msg in messages {
            let to = msg.get_to();
            if to == self.local_node_id {
                continue;
            }
            debug!("send raft message:{:?}, to:{}", msg, to);
            let data: Vec<u8> = raftPreludeMessage::encode_to_vec(&msg);
            if let Some(node) = self.cache_placement.get_votes_node_by_id(to) {
                match self
                    .peer_message_send
                    .send(PeerMessage {
                        to: node.node_addr,
                        data,
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(CommonError::CommmonError(format!(
                            "Failed to write Raft Message to send queue with error message: {:?}",
                            e.to_string()
                        )))
                    }
                }
            } else {
                return Err(CommonError::CommmonError(format!("raft message was sent to node {}, but the node information could not be found. It may be that the node is not online yet.",to)));
            }
        }
        Ok(())
    }

    fn new_node(&self) -> Result<RawNode<RaftRocksDBStorage>, CommonError> {
        let storage = RaftRocksDBStorage::new(self.raft_storage.clone());
        let core = storage.read_lock()?;

        // recover hard commit
        let hs = core.hard_state();
        let conf = self.build_config(hs.commit);

        // update cs voetes
        let mut cs = core.conf_state();
        let vote_ids = self
            .cache_placement
            .get_raft_votes()
            .iter()
            .map(|node| node.node_id)
            .collect();
        cs.voters = vote_ids;
        let _ = core.save_conf_state(cs);

        // init config
        let logger = self.build_slog();
        let storage = RaftRocksDBStorage::new(self.raft_storage.clone());

        let node = match RawNode::new(&conf, storage, &logger) {
            Ok(data) => data,
            Err(e) => {
                return Err(CommonError::CommmonError(e.to_string()));
            }
        };
        Ok(node)
    }

    fn build_config(&self, apply: u64) -> Config {
        let conf = placement_center_conf();
        Config {
            // The unique ID for the Raft node.
            // id: self.config.node_id,
            id: conf.node.node_id,
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
            .truncate(true)
            .open(path)
            .unwrap();

        let decorator = slog_term::PlainDecorator::new(file);
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain)
            .chan_size(4096)
            .overflow_strategy(slog_async::OverflowStrategy::Block)
            .build()
            .fuse();

        slog::Logger::root(drain, o!("tag" => format!("meta-node-id={}", 1)))
    }

    fn try_record_role_change(&self, raft_node: &RawNode<RaftRocksDBStorage>) {
        if self
            .cache_placement
            .is_raft_role_change(raft_node.raft.state)
        {
            info!(
                "Node Raft Role changes from  【{:?}】 to 【{:?}】",
                self.cache_placement.get_current_raft_role(),
                raft_node.raft.state
            );
            self.cache_placement
                .update_raft_role(raft_node.raft.state, raft_node.raft.leader_id);
        }
    }
}
