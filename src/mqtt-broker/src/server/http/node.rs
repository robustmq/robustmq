use axum::extract::State;
use bytes::Bytes;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, http_response::success_response, metrics::dump_metrics,
};
use protocol::mqtt::{MQTTPacket, Publish, PublishProperties};
use crate::server::tcp::packet::ResponsePackage;
use super::server::HttpServerState;

pub async fn metrics() -> String {
    return dump_metrics();
}

pub async fn hearbeat_info(State(state): State<HttpServerState>) -> String {
    let data = state.heartbeat_manager.read().await;
    return success_response(data.heartbeat_data.clone());
}

pub async fn metadata_info(State(state): State<HttpServerState>) -> String {
    let data = state.metadata_cache.read().await;
    return success_response(data.clone());
}

pub async fn subscribe_info(State(state): State<HttpServerState>) -> String {
    let data = state.subscribe_manager.read().await;
    return success_response(data.clone());
}

pub async fn index(State(state): State<HttpServerState>) -> String {
    let conf = broker_mqtt_conf();
    return success_response(conf.clone());
}

pub async fn test_subscribe_pub(State(state): State<HttpServerState>) -> String {
    let sub_manager = state.subscribe_manager.read().await;
    for (connect_id, sub) in sub_manager.subscribe_list.clone() {
        let publish = Publish {
            dup: false,
            qos: protocol::mqtt::QoS::AtLeastOnce,
            pkid: sub.packet_identifier,
            retain: false,
            topic: Bytes::from("loboxu/test".to_string()),
            payload: Bytes::from("subscribe loboxu success".to_string()),
        };

        let mut properties = PublishProperties::default();
        properties.user_properties = vec![("key1".to_string(), "val1".to_string())];
        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::Publish(publish, Some(properties)),
        };
        state.response_queue_sx5.send(resp).unwrap();
    }

    return success_response("");
}
