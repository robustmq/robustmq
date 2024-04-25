use super::server::HttpServerState;
use crate::metadata::{cluster::Cluster, session::Session, topic::Topic, user::User};
use axum::extract::State;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, http_response::success_response, metrics::dump_metrics,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

pub async fn metrics() -> String {
    return dump_metrics();
}

pub async fn hearbeat_info<T>(State(state): State<HttpServerState<T>>) -> String {
    let data = state.heartbeat_manager.shard_data.clone();
    return success_response(data);
}

pub async fn metadata_info<T>(State(state): State<HttpServerState<T>>) -> String {
    let result = MetadataCacheResult {
        cluster_info: state.metadata_cache.cluster_info.clone(),
        user_info: state.metadata_cache.user_info.clone(),
        session_info: state.metadata_cache.session_info.clone(),
        topic_info: state.metadata_cache.topic_info.clone(),
        topic_id_name: state.metadata_cache.topic_id_name.clone(),
        connect_id_info: state.metadata_cache.connect_id_info.clone(),
        login_info: state.metadata_cache.login_info.clone(),
    };

    return success_response(result);
}

pub async fn subscribe_info<T>(State(state): State<HttpServerState<T>>) -> String {
    return success_response(state.subscribe_manager.topic_subscribe.clone());
}

pub async fn index<T>(State(state): State<HttpServerState<T>>) -> String {
    let conf = broker_mqtt_conf();
    return success_response(conf.clone());
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MetadataCacheResult {
    pub cluster_info: DashMap<String, Cluster>,
    pub user_info: DashMap<String, User>,
    pub session_info: DashMap<String, Session>,
    pub topic_info: DashMap<String, Topic>,
    pub topic_id_name: DashMap<String, String>,
    pub connect_id_info: DashMap<u64, String>,
    pub login_info: DashMap<u64, bool>,
}
