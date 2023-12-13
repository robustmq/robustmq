use common::config::meta::MetaConfig;
use raft::Config;
use raft_proto::eraftpb::Snapshot;

use crate::storage::raft::RaftRocksDBStorage;

use super::node::Node;

pub struct MetaRaft {
    config: MetaConfig,
    leader: Node,
}

impl MetaRaft {
    pub fn new(config: MetaConfig, leader: Node,) -> Self {
        return Self { config, leader };
    }

    pub async fn run(&self) {
        if self.config.node_id == self.leader.leader_id.unwrap() {
            self.new_leader();
        } else {
            self.new_follower()
        }
    }

    pub fn new_leader(&self) {
        let conf = self.build_config();
        let mut s  = Snapshot::default();

        // Because we don't use the same configuration to initialize every node, so we use
        // a non-zero index to force new followers catch up logs by snapshot first, which will
        // bring all nodes to the same initial state.
        s.mut_metadata().index = 1;
        s.mut_metadata().term = 1;
        s.mut_metadata().mut_conf_state().voters = vec![self.config.node_id];

        let mut storage = RaftRocksDBStorage::new(&self.config);
        
    }

    pub fn new_follower(&self) {}

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
}
