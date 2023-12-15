use std::fmt::{self, Display};

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
pub struct Node {
    pub node_ip: String,
    pub node_id: u64,
    pub leader_id: Option<u64>,
    pub leader_ip: Option<String>,
    pub raft_state: NodeRaftState,
    pub state: NodeState,
    pub voter: Option<u64>,
}

impl Node {
    pub fn new(node_ip: String, node_id: u64) -> Node {
        Node {
            node_ip,
            node_id,
            leader_id: None,
            leader_ip: None,
            raft_state: NodeRaftState::Candidate,
            state: NodeState::Starting,
            voter: None,
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "node_ip:{},node_id:{},leader_ip:{:?},leader_id:{:?}",
            self.node_ip, self.node_id, self.leader_ip, self.leader_id
        )
    }
}

