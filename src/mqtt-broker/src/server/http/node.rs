use std::collections::HashMap;

use crate::metadata::{
    cluster::Cluster, session::Session, subscriber::Subscriber, topic::Topic, user::User,
};

use super::server::HttpServerState;
use axum::extract::State;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, http_response::success_response, metrics::dump_metrics,
};
use serde::{Deserialize, Serialize};

pub async fn metrics() -> String {
    return dump_metrics();
}

pub async fn hearbeat_info(State(state): State<HttpServerState>) -> String {
    let data = state.heartbeat_manager.read().await;
    return success_response(data.heartbeat_data.clone());
}

pub async fn metadata_info(State(state): State<HttpServerState>) -> String {
    let data = state.metadata_cache.read().await;
    let result = MetadataCacheResult {
        cluster_info: data.cluster_info.clone(),
        user_info: data.user_info.clone(),
        session_info: data.session_info.clone(),
        topic_info: data.topic_info.clone(),
        topic_id_name: data.topic_id_name.clone(),
        connect_id_info: data.connect_id_info.clone(),
        login_info: data.login_info.clone(),
    };

    return success_response(result);
}

pub async fn subscribe_info(State(state): State<HttpServerState>) -> String {
    let data = state.subscribe_manager.read().await;
    return success_response(data.topic_subscribe.clone());
}

pub async fn index(State(state): State<HttpServerState>) -> String {
    let conf = broker_mqtt_conf();
    return success_response(conf.clone());
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MetadataCacheResult {
    pub cluster_info: Cluster,
    pub user_info: HashMap<String, User>,
    pub session_info: HashMap<String, Session>,
    pub topic_info: HashMap<String, Topic>,
    pub topic_id_name: HashMap<String, String>,
    pub connect_id_info: HashMap<u64, String>,
    pub login_info: HashMap<u64, bool>,
}
