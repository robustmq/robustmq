use crate::{
    raft::peer::{self, PeerMessage, PeersManager},
    Node,
};
use ahash::AHashMap;
use common::log::{error_meta, info_meta};
use tokio::sync::mpsc::Sender;
use toml::Table;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum NodeState {
    Running,
    Starting,
    Stoping,
    Stop,
}

#[derive(Clone)]
pub struct Cluster {
    pub local: Node,
    pub leader: Option<Node>,
    pub role: NodeRole,
    pub state: NodeState,
    pub peers: AHashMap<u64, Node>,
    peers_send: Sender<PeerMessage>,
}

impl Cluster {
    pub fn new(local: Node, peers_send: Sender<PeerMessage>, nodes: Table) -> Cluster {
        let mut peers = AHashMap::new();
        for (node_id, addr) in nodes {
            let (ip, port) = addr.as_str().unwrap().split_once(":").unwrap();
            let p: u16 = port.to_string().trim().parse().unwrap();
            let id: u64 = node_id.to_string().trim().parse().unwrap();
            peers.insert(id, Node::new(ip.to_string(), id, p));
        }

        Cluster {
            local,
            leader: None,
            role: NodeRole::Candidate,
            state: NodeState::Starting,
            peers: peers,
            peers_send,
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

    pub async fn send_message(&mut self, id: u64, msg: Vec<u8>) {
        if let Some(node) = self.get_node_by_id(id) {
            match self
                .peers_send
                .send(PeerMessage {
                    to: node.addr(),
                    data: msg,
                })
                .await
            {
                Ok(_) => {}
                Err(e) => error_meta(&format!(
                    "Failed to write Raft Message to send queue with error message: {:?}",
                    e.to_string()
                )),
            }
        } else {
            error_meta(&format!("raft message was sent to node {}, but the node information could not be found. It may be that the node is not online yet.",id));
        }
    }

    pub fn is_leader(&self) -> bool {
        self.role == NodeRole::Leader
    }

    pub fn set_role(&mut self, role: NodeRole) {
        self.role = role;
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
}
