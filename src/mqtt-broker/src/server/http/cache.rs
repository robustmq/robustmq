use std::collections::HashMap;

use super::server::HttpServerState;
use crate::{
    core::heartbeat_cache::HeartbeatShard,
    metadata::{
        cluster::Cluster,
        connection::Connection,
        session::Session,
        subscriber::{SubscribeData, Subscriber},
    },
    qos::{PublishQosMessageData, QosData},
    subscribe::subscribe_cache::{ShareLeaderSubscribeData, ShareSubShareSub},
};
use axum::extract::State;
use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    http_response::success_response,
    metrics::dump_metrics,
};
use dashmap::DashMap;
use metadata_struct::mqtt::{topic::MQTTTopic, user::MQTTUser};
use serde::{Deserialize, Serialize};

pub async fn metrics() -> String {
    return dump_metrics();
}

pub async fn cache_info(State(state): State<HttpServerState>) -> String {
    let result = MetadataCacheResult {
        config: broker_mqtt_conf().clone(),

        cluster_name: state.metadata_cache.cluster_name.clone(),
        cluster_info: state.metadata_cache.cluster_info.clone(),
        user_info: state.metadata_cache.user_info.clone(),
        session_info: state.metadata_cache.session_info.clone(),
        connection_info: state.metadata_cache.connection_info.clone(),
        topic_info: state.metadata_cache.topic_info.clone(),
        topic_id_name: state.metadata_cache.topic_id_name.clone(),
        subscribe_filter: state.metadata_cache.subscribe_filter.clone(),
        publish_pkid_info: state.metadata_cache.publish_pkid_info.clone(),

        heartbeat_data: state.heartbeat_manager.shard_data.clone(),

        exclusive_subscribe: state.subscribe_cache.exclusive_subscribe.clone(),
        share_leader_subscribe: state.subscribe_cache.share_leader_subscribe.clone(),
        share_follower_subscribe: state.subscribe_cache.share_follower_subscribe.clone(),
        share_follower_identifier_id: state.subscribe_cache.share_follower_identifier_id.clone(),

        exclusive_push_thread: state.subscribe_cache.exclusive_push_thread_keys(),
        share_leader_push_thread: state.subscribe_cache.share_leader_push_thread_keys(),
        share_follower_resub_thread: state.subscribe_cache.share_follower_resub_thread_keys(),

        qos_pkid_data: state.qos_memory.qos_pkid_data.clone(),
        sub_pkid_data: state.qos_memory.sub_pkid_data.clone(),
    };

    return success_response(result);
}

pub async fn index() -> String {
    let mut cluster: HashMap<String, String> = HashMap::new();
    cluster.insert("version".to_string(), "1.0.0".to_string());
    return success_response(cluster);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MetadataCacheResult {
    // config
    pub config: BrokerMQTTConfig,

    // metadata_cache
    pub cluster_name: String,
    pub cluster_info: DashMap<String, Cluster>,
    pub user_info: DashMap<String, MQTTUser>,
    pub session_info: DashMap<String, Session>,
    pub connection_info: DashMap<u64, Connection>,
    pub topic_info: DashMap<String, MQTTTopic>,
    pub topic_id_name: DashMap<String, String>,
    pub subscribe_filter: DashMap<String, DashMap<String, SubscribeData>>,
    pub publish_pkid_info: DashMap<String, Vec<u16>>,

    // heartbeat data
    pub heartbeat_data: DashMap<u64, HeartbeatShard>,

    // subscribe data
    pub exclusive_subscribe: DashMap<String, Subscriber>,
    pub share_leader_subscribe: DashMap<String, ShareLeaderSubscribeData>,
    pub share_follower_subscribe: DashMap<String, ShareSubShareSub>,
    pub share_follower_identifier_id: DashMap<usize, String>,

    pub exclusive_push_thread: Vec<String>,
    pub share_leader_push_thread: Vec<String>,
    pub share_follower_resub_thread: Vec<String>,

    // QosMemory
    pub qos_pkid_data: DashMap<String, QosData>,
    pub sub_pkid_data: DashMap<String, u64>,
}
