use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Node;

#[derive(Serialize, Deserialize)]
pub struct IndexResponse {
    pub local: Node,
    pub node_lists: HashMap<u64, Node>,
    pub raft: RaftInfo,
}

#[derive(Serialize, Deserialize)]
pub struct RaftInfo {
    pub role: String,
    pub first_index: u64,
    pub last_index: u64,
    pub term: u64,
    pub vote: u64,
    pub commit: u64,
    pub voters: Vec<u64>,
    pub learners: Vec<u64>,
    pub voters_outgoing: Vec<u64>,
    pub learners_next: Vec<u64>,
    pub auto_leave: bool,
    pub uncommit_index:HashMap<u64, i8>,
}

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub code: u64,
    pub data: T,
}

pub fn success_response<T: Serialize>(data: T) -> String {
    let resp = Response {
        code: 0,
        data: data,
    };
    return serde_json::to_string(&resp).unwrap();
}

pub fn error_response() -> String {
    return "".to_string();
}
