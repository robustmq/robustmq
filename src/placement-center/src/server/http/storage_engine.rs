use super::server::HttpServerState;
use axum::extract::State;
use common_base::http_response::success_response;

pub async fn clusters(State(state): State<HttpServerState>) -> String {
    return success_response(state.engine_cache.shard_list.clone());
}


pub async fn storage_engine(State(state): State<HttpServerState>) -> String {
    return success_response(state.engine_cache.shard_list.clone());
}

pub async fn shard_list(State(state): State<HttpServerState>) -> String {
    return success_response(state.engine_cache.shard_list.clone());
}

pub async fn shard_info(State(state): State<HttpServerState>) -> String {
    return success_response(state.engine_cache.shard_list.clone());
}
