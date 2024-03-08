use std::collections::HashMap;

use crate:: Node;
use common::log:: info_meta;
use raft::StateRole;
use toml::Table;

#[derive(PartialEq, Default, Debug,Eq, PartialOrd, Ord, Clone)]
pub enum NodeState {
    #[default]
    Running,
    Starting,
    Stoping,
    Stop,
}

#[derive(Clone, Default, Debug)]
pub struct PlacementClusterCache {
    pub local: Node,
    pub leader: Option<Node>,
    pub state: NodeState,
    pub raft_role: StateRole,
    pub peers: HashMap<u64, Node>,
}


impl PlacementClusterCache {
    pub fn new(local: Node, nodes: Table) -> PlacementClusterCache {
        let mut peers = HashMap::new();
        for (node_id, addr) in nodes {
            let (ip, port) = addr.as_str().unwrap().split_once(":").unwrap();
            let p: u16 = port.to_string().trim().parse().unwrap();
            let id: u64 = node_id.to_string().trim().parse().unwrap();
            peers.insert(id, Node::new(ip.to_string(), id, p));
        }

        PlacementClusterCache {
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
            return leader.addr();
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
