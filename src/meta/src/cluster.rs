use crate::{
    raft::peer::{PeerManager, PeerMessage},
    Node,
};
use common::log::{error_meta, info_meta};

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
    pub peer_manager: PeerManager,
}

impl Cluster {
    pub fn new(local: Node) -> Cluster {
        Cluster {
            local,
            leader: None,
            role: NodeRole::Candidate,
            state: NodeState::Starting,
            peer_manager: PeerManager::new(3),
        }
    }

    // Add Meta node
    pub fn add_peer(&mut self, id: u64, node: Node) {
        info_meta(&format!("addd node,{:?}", node));
        self.peer_manager.add_peer(id, node);
    }

    // Add Meta node
    pub fn remove_peer(&mut self, id: u64) {
        self.peer_manager.remove_peer(id);
    }

    pub fn send_message(&mut self, id: u64, msg: Vec<u8>) {
        info_meta(&format!("send to:{}", id));
        if let Some(node) = self.peer_manager.get_node_by_id(id) {
            self.peer_manager.send_message(PeerMessage {
                to: node.addr(),
                data: msg,
            })
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

    pub fn start_process_thread(&self) {
        self.peer_manager.start();
    }
}
