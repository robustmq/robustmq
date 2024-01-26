use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Node;

#[derive(Serialize, Deserialize)]
pub struct IndexResponse {
    pub local: Node,
    pub node_lists: HashMap<u64, Node>,
    pub role: String,
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
