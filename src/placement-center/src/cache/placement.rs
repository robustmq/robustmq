use common_base::log::info_meta;
use raft::StateRole;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};
use toml::Table;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Node {
    pub ip: String,
    pub id: u64,
    pub inner_port: u16,
}

impl Node {
    pub fn new(ip: String, id: u64, port: u16) -> Node {
        Node {
            ip,
            id,
            inner_port: port,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.inner_port)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ip:{},id:{},port:{}", self.ip, self.id, self.inner_port)
    }
}

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
    pub fn new(local: Node, nodes: Table) -> PlacementCache {
        let mut peers = HashMap::new();
        for (node_id, addr) in nodes {
            let (ip, port) = addr.as_str().unwrap().split_once(":").unwrap();
            let p: u16 = port.to_string().trim().parse().unwrap();
            let id: u64 = node_id.to_string().trim().parse().unwrap();
            peers.insert(id, Node::new(ip.to_string(), id, p));
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
