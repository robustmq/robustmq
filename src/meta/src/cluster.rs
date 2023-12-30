use std::collections::HashMap;

use futures_util::task::SpawnExt;

use crate::{raft::peer::Peer, Node};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum NodeRaftState {
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
    pub raft_state: NodeRaftState,
    pub state: NodeState,
    pub peers: HashMap<u64, Peer>,
}

impl Cluster {
    pub fn new(local: Node) -> Cluster {
        Cluster {
            local,
            leader: None,
            raft_state: NodeRaftState::Candidate,
            state: NodeState::Starting,
            peers: HashMap::new(),
        }
    }

    // Add Meta node
    pub fn add_peer(&mut self, id: u64, peer: Peer) {
        if let Some(mut pr) = self.peers.insert(id, peer) {
            tokio::spawn(async move {
                pr.start().await;
            });
        }
    }

    // Add Meta node
    pub fn remove_peer(&mut self, id: u64) {
        if let Some(mut per) = self.peers.remove(&id) {
            per.stop();
        }
    }

    pub fn send_message(&mut self, id: u64, msg: Vec<u8>) {
        if let Some(per) = self.peers.get_mut(&id) {
            per.push(msg);
        }
    }
}
