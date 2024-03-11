use super::{response::success_response, server::HttpServerState};
use axum::extract::State;

pub async fn storage_engine(State(state): State<HttpServerState>) -> String {
    let data = state.engine_cache.read().unwrap();
    return success_response(data.clone());
}


pub async fn shard_list(State(state): State<HttpServerState>) -> String {
    let data = state.engine_cache.read().unwrap();
    return success_response(data.clone());
}

pub async fn shard_info(State(state): State<HttpServerState>) -> String {
    let data = state.engine_cache.read().unwrap();
    return success_response(data.clone());
}
