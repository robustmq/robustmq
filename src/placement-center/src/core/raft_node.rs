use serde::{Deserialize, Serialize};

#[derive(PartialEq, Default, Debug, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum RaftNodeState {
    #[default]
    Running,
    Starting,
    Stoping,
    Stop,
}

#[derive(PartialEq, Default, Debug, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct RaftNode {
    pub node_id: u64,
    pub node_addr: String,
}
