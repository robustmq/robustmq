use super::message::RaftMessage;
use super::node::Node;
use crate::storage::raft_storage::RaftRocksDBStorage;
use common::config::meta::MetaConfig;
use raft::prelude::Message as raftPreludeMessage;
use raft::{Config, RawNode};
use raft_proto::eraftpb::{ConfChange, Snapshot};
use raft_proto::eraftpb::{Entry, EntryType};
use slog::o;
use slog::Drain;
use std::fs::OpenOptions;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;

pub struct MetaRaft {
    config: MetaConfig,
    leader: Node,
    receiver: Receiver<RaftMessage>,
}

impl MetaRaft {
    pub fn new(config: MetaConfig, leader: Node, receiver: Receiver<RaftMessage>) -> Self {
        return Self {
            config: config,
            leader: leader,
            receiver: receiver,
        };
    }

    pub async fn run(&mut self) {
        let mut raft_node = if self.config.node_id == self.leader.node_id {
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
                Err(_) => break,
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

        // After receiving the data sent by the client,
        // the data needs to be sent to other Raft nodes for persistent storage.
        if !ready.messages().is_empty() {
            self.send_message(ready.take_messages());
        }

        // If the snapshot is not empty, save the snapshot to Storage, and apply
        // the data in the snapshot to the State Machine asynchronously.
        // (Although synchronous apply can also be applied here,
        // but the snapshot is usually large. Synchronization blocks threads).
        if *ready.snapshot() != Snapshot::default() {
            let s = ready.snapshot().clone();
            raft_node.mut_store().apply_snapshot(s).unwrap();
        }

        // The committed raft log can be applied to the State Machine.
        self.handle_committed_entries(ready.take_committed_entries());

        // messages need to be stored to Storage before they can be sent.Save entries to Storage.
        if !ready.entries().is_empty() {
            let entries = ready.entries();
            raft_node.mut_store().append(entries).unwrap();
        }

        // If there is a change in HardState, such as a revote,
        // term is increased, the hs will not be empty.Persist non-empty hs.
        if let Some(hs) = ready.hs() {
            raft_node.mut_store().set_hard_state(hs).unwrap();
        }

        // If SoftState changes, such as adding or removing nodes, ss will not be empty.
        // persist non-empty ss.
        if let Some(ss) = ready.ss() {}

        //
        if !ready.persisted_messages().is_empty() {
            self.send_message(ready.take_persisted_messages());
        }

        // A call to advance tells Raft that it is ready for processing.
        let mut light_rd = raft_node.advance(ready);
        if let Some(commit) = light_rd.commit_index() {
            raft_node.mut_store().set_hard_state_comit(commit).unwrap();
        }

        self.send_message(light_rd.take_messages());

        self.handle_committed_entries(light_rd.take_committed_entries());

        raft_node.advance_apply();
    }

    fn handle_committed_entries(&self, entrys: Vec<Entry>) {
        for entry in entrys {
            if entry.data.is_empty() {
                continue;
            }
            if let EntryType::EntryConfChange = entry.get_entry_type() {
                let mut cc = ConfChange::default();

                self.handle_config_change();
            } else {
                self.handle_normal();
            }
        }
    }

    fn handle_config_change(&self) {}

    fn handle_normal(&self) {}

    fn send_message(&self, messages: Vec<raftPreludeMessage>) {
        for msg in messages  {

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
        RawNode::new(&conf, storage, &logger).unwrap()
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
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain)
            .chan_size(4096)
            .overflow_strategy(slog_async::OverflowStrategy::Block)
            .build()
            .fuse();
        let logger = slog::Logger::root(drain, o!("tag" => format!("[{}]", 1)));
        return logger;
    }
}
