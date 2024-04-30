use common_base::{
    config::placement_center::placement_center_conf, log::info_meta, tools::get_local_ip,
};
use protocol::placement_center::generate::common::ClusterType;
use raft::StateRole;

use std::collections::HashMap;
use toml::Table;

use crate::structs::node::Node;

#[derive(PartialEq, Default, Debug, Eq, PartialOrd, Ord, Clone)]
pub enum NodeState {
    #[default]
    Running,
    Starting,
    Stoping,
    Stop,
}

#[derive(Clone, Default, Debug)]
pub struct PlacementCache {
    pub local: Node,
    pub leader: Option<Node>,
    pub state: NodeState,
    pub raft_role: StateRole,
    pub peers: HashMap<u64, Node>,
}

impl PlacementCache {
    pub fn new() -> PlacementCache {
        let config = placement_center_conf();
        let mut local = Node::default();
        local.cluster_type = ClusterType::PlacementCenter.as_str_name().to_string();
        local.cluster_name = config.cluster_name.clone();
        local.node_inner_addr = format!("{}:{}", config.addr.clone(), config.grpc_port);
        local.node_ip = config.addr.clone();
        local.node_id = config.node_id;

        let mut peers = HashMap::new();
        for (node_id, addr) in config.nodes.clone() {
            let (ip, _) = addr.as_str().unwrap().split_once(":").unwrap();
            let id: u64 = node_id.to_string().trim().parse().unwrap();
            let mut node = Node::default();

            node.cluster_type = ClusterType::PlacementCenter.as_str_name().to_string();
            node.cluster_name = config.cluster_name.clone();
            node.node_inner_addr = addr.to_string();
            node.node_ip = ip.to_string();
            node.node_id = id;
            peers.insert(id, node);
        }

        PlacementCache {
            local,
            leader: None,
            raft_role: StateRole::Follower,
            state: NodeState::Starting,
            peers: peers,
        }
    }

    // Add Meta node
    pub fn add_peer(&mut self, id: u64, node: Node) {
        info_meta(&format!("add peer node:{:?}", node));
        self.peers.insert(id, node);
    }

    // Add Meta node
    pub fn remove_peer(&mut self, id: u64) {
        info_meta(&format!("remove peer node id:{:?}", id));
        self.peers.remove(&id);
    }

    pub fn is_leader(&self) -> bool {
        self.raft_role == StateRole::Leader
    }

    pub fn set_role(&mut self, role: StateRole) {
        self.raft_role = role;
    }

    pub fn set_leader(&mut self, leader: Node) {
        self.leader = Some(leader);
    }

    pub fn get_node_by_id(&self, id: u64) -> Option<&Node> {
        self.peers.get(&id)
    }

    pub fn node_ids(&self) -> Vec<u64> {
        let mut voters = Vec::new();
        for (id, _) in self.peers.iter() {
            voters.push(*id);
        }
        return voters;
    }

    pub fn leader_addr(&self) -> String {
        if let Some(leader) = self.leader.clone() {
            return leader.node_inner_addr;
        }
        return "".to_string();
    }

    pub fn leader_alive(&self) -> bool {
        if let Some(_) = self.leader.clone() {
            return true;
        }
        return false;
    }
}
