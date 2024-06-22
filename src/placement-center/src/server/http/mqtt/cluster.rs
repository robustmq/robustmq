use axum::extract::State;

use crate::server::http::server::HttpServerState;

pub async fn cluster_get(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}

pub async fn cluster_set(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}

pub async fn cluster_list(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}
