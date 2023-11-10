#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeRaftState {
    Leader,
    Follower,
    Candidate,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeState {
    Running,
    Starting,
    Stoping,
    Stop,
}

pub struct Node {
    pub node_ip: String,
    pub node_id: i32,
    pub leader_id: Option<i32>,
    pub leader_ip: Option<String>,
    pub raft_state: NodeRaftState,
    pub state: NodeState,
    pub voter: Option<i32>,
}

impl Node {
    pub fn new(node_ip: String, node_id: i32) -> Node {
        Node {
            node_ip: node_ip,
            node_id: node_id,
            leader_id: None,
            leader_ip: None,
            raft_state: NodeRaftState::Candidate,
            state: NodeState::Starting,
            voter: None,
        }
    }

    pub fn update_raft_state(&mut self, raft_state: NodeRaftState) {
        self.raft_state = raft_state
    }

    pub fn update_status(&mut self, state: NodeState) {
        self.state = state;
    }
}
