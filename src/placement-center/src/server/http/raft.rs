use super::server::HttpServerState;
use crate::raftv2::{raft_node::Node, typeconfig::TypeConfig};
use axum::extract::State;
use common_base::http_response::{error_response, success_response};
use openraft::{error::Infallible, RaftMetrics};
use std::collections::{BTreeMap, BTreeSet};

pub async fn add_leadrner(State(state): State<HttpServerState>) -> String {
    let node_id = 3;
    let node = Node {
        rpc_addr: "127.0.0.0:7654".to_string(),
        node_id: 2,
    };
    match state
        .placement_center_storage
        .openraft_node
        .add_learner(node_id, node, true)
        .await
    {
        Ok(data) => {
            return success_response(data);
        }
        Err(e) => {
            return error_response(e.to_string());
        }
    }
}

pub async fn change_membership(State(state): State<HttpServerState>) -> String {
    let mut body = BTreeSet::new();
    body.insert(3);
    match state
        .placement_center_storage
        .openraft_node
        .change_membership(body, true)
        .await
    {
        Ok(data) => {
            return success_response(data);
        }
        Err(e) => {
            return error_response(e.to_string());
        }
    }
}

pub async fn init(State(state): State<HttpServerState>) -> String {
    let node_id = 3;
    let node = Node {
        rpc_addr: "127.0.0.0:7654".to_string(),
        node_id: 2,
    };

    let mut nodes = BTreeMap::new();
    nodes.insert(node_id, node);

    match state
        .placement_center_storage
        .openraft_node
        .initialize(nodes)
        .await
    {
        Ok(data) => {
            return success_response(data);
        }
        Err(e) => {
            return error_response(e.to_string());
        }
    }
}

pub async fn raft_status(State(state): State<HttpServerState>) -> String {
    let metrics = state
        .placement_center_storage
        .openraft_node
        .metrics()
        .borrow()
        .clone();
    let res: Result<RaftMetrics<TypeConfig>, Infallible> = Ok(metrics);
    return success_response(res);
}
