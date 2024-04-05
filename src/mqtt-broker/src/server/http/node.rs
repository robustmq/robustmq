use axum::extract::State;
use bytes::Bytes;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, http_response::success_response, metrics::dump_metrics,
};
use protocol::mqtt::{MQTTPacket, Publish};

use crate::server::tcp::packet::ResponsePackage;

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

pub async fn subscribe_info(State(state): State<HttpServerState>) -> String {
    let data = state.subscribe_manager.read().unwrap();
    return success_response(data.subscribe_list.clone());
}

pub async fn index(State(state): State<HttpServerState>) -> String {
    let conf = broker_mqtt_conf();
    return success_response(conf.clone());
}

pub async fn test_subscribe_pub(State(state): State<HttpServerState>) -> String {
    let publish = Publish {
        dup: true,
        qos: protocol::mqtt::QoS::AtLeastOnce,
        retain: true,
        topic: Bytes::from("/loboxu/test".to_string()),
        pkid: 1,
        payload: Bytes::from("lobo success".to_string()),
    };
    let packet = MQTTPacket::Publish(publish, None);
    let resp = ResponsePackage {
        connection_id: 1,
        packet,
    };
    state.response_queue_sx.send(resp).unwrap();
    return success_response("");
}
