use common::config::meta::MetaConfig;

use super::node::Node;

pub struct MetaRaft {
    config: MetaConfig,
    leader: Node,
}

impl MetaRaft {
    pub fn new(config: MetaConfig, leader: Node) -> Self {
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

    }

    pub fn new_follower(&self) {

    }
}
