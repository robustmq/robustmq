use axum::extract::State;
use common_base::{config::broker_mqtt::broker_mqtt_conf, http_response::success_response, metrics::dump_metrics};

use super::server::HttpServerState;

pub async fn metrics() -> String {
    return dump_metrics();
}

pub async fn hearbeat_info(State(state): State<HttpServerState>) -> String {
    let data = state.heartbeat_manager.read().unwrap();
    return success_response(data.clone());
}


pub async fn metadata_info(State(state): State<HttpServerState>) -> String {
    let data = state.metadata_cache.read().unwrap();
    return success_response(data.clone());
}


pub async fn index(State(state): State<HttpServerState>) -> String {
    let conf = broker_mqtt_conf();
    return success_response(conf.clone());
}