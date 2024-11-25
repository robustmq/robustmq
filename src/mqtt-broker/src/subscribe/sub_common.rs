// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::Message;
use bytes::BytesMut;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use common_base::error::mqtt_broker::MqttBrokerError;
use grpc_clients::placement::mqtt::call::placement_get_share_sub_leader;
use grpc_clients::pool::ClientPool;
use log::error;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{MqttPacket, MqttProtocol, PubRel, Publish, PublishProperties, QoS};
use protocol::placement_center::placement_center_mqtt::{
    GetShareSubLeaderReply, GetShareSubLeaderRequest,
};
use regex::Regex;
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::{sleep, timeout};

use crate::handler::cache::{CacheManager, QosAckPackageData};
use crate::observability::metrics::packets::record_sent_metrics;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::ResponsePackage;
use crate::storage::message::MessageStorage;

const SHARE_SUB_PREFIX: &str = "$share";

pub fn path_contain_sub(_: &str) -> bool {
    true
}

pub fn sub_path_validator(sub_path: String) -> bool {
    let regex = Regex::new(r"^[\$a-zA-Z0-9_#+/]+$").unwrap();

    if !regex.is_match(&sub_path) {
        return false;
    }

    for path in sub_path.split("/") {
        if path.contains("+") && path != "+" {
            return false;
        }
        if path.contains("#") && path != "#" {
            return false;
        }
    }

    true
}

pub fn path_regex_match(topic_name: String, sub_path: String) -> bool {
    let path = if is_share_sub(sub_path.clone()) {
        let (_, group_path) = decode_share_info(sub_path);
        group_path
    } else {
        sub_path
    };

    // Path perfect matching
    if topic_name == path {
        return true;
    }

    if path.contains("+") {
        let sub_regex = path.replace("+", "[^+*/]+");
        let re = Regex::new(&sub_regex.to_string()).unwrap();
        return re.is_match(&topic_name);
    }

    if path.contains("#") {
        if path.split("/").last().unwrap() != "#" {
            return false;
        }
        let sub_regex = path.replace("#", "[^+#]+");
        let re = Regex::new(&sub_regex.to_string()).unwrap();
        return re.is_match(&topic_name);
    }

    false
}

pub fn min_qos(qos: QoS, sub_qos: QoS) -> QoS {
    if qos <= sub_qos {
        return qos;
    }
    sub_qos
}

pub async fn get_sub_topic_id_list(
    metadata_cache: Arc<CacheManager>,
    sub_path: String,
) -> Vec<String> {
    let mut result = Vec::new();
    for (topic_id, topic_name) in metadata_cache.topic_id_name.clone() {
        if path_regex_match(topic_name.clone(), sub_path.clone()) {
            result.push(topic_id);
        }
    }
    result
}

pub fn is_share_sub(sub_name: String) -> bool {
    sub_name.starts_with(SHARE_SUB_PREFIX)
}

pub fn decode_share_info(sub_name: String) -> (String, String) {
    let mut str_slice: Vec<&str> = sub_name.split("/").collect();
    str_slice.remove(0);
    let group_name = str_slice.remove(0).to_string();
    let sub_name = format!("/{}", str_slice.join("/"));
    (group_name, sub_name)
}

pub async fn get_share_sub_leader(
    client_pool: Arc<ClientPool>,
    group_name: String,
) -> Result<GetShareSubLeaderReply, CommonError> {
    let conf = broker_mqtt_conf();
    let req = GetShareSubLeaderRequest {
        cluster_name: conf.cluster_name.clone(),
        group_name,
    };
    match placement_get_share_sub_leader(client_pool, &conf.placement_center, req).await {
        Ok(reply) => Ok(reply),
        Err(e) => Err(e),
    }
}

pub async fn wait_packet_ack(sx: &Sender<QosAckPackageData>) -> Option<QosAckPackageData> {
    let res = timeout(Duration::from_secs(120), async {
        match sx.subscribe().recv().await {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    });

    (res.await).unwrap_or_default()
}

pub async fn publish_message_to_client(
    resp: ResponsePackage,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), CommonError> {
    if let Some(protocol) = connection_manager.get_connect_protocol(resp.connection_id) {
        record_sent_metrics(&resp, connection_manager);

        let response: MqttPacketWrapper = MqttPacketWrapper {
            protocol_version: protocol.clone().into(),
            packet: resp.packet,
        };

        if connection_manager.is_websocket(resp.connection_id) {
            let mut codec = MqttCodec::new(Some(protocol.into()));
            let mut buff = BytesMut::new();
            match codec.encode_data(response, &mut buff) {
                Ok(()) => {}
                Err(e) => {
                    error!("Websocket encode back packet failed with error message: {e:?}");
                }
            }
            return connection_manager
                .write_websocket_frame(resp.connection_id, Message::Binary(buff.to_vec()))
                .await;
        }
        return connection_manager
            .write_tcp_frame(resp.connection_id, response)
            .await;
    }
    Ok(())
}

pub async fn qos2_send_publish(
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<CacheManager>,
    client_id: &str,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
    stop_sx: &broadcast::Sender<bool>,
) -> Result<(), MqttBrokerError> {
    let mut retry_times = 0;
    let mut stop_rx = stop_sx.subscribe();
    loop {
        let connect_id = if let Some(id) = metadata_cache.get_connect_id(client_id) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        if let Some(conn) = metadata_cache.get_connection(connect_id) {
            if publish.payload.len() > (conn.max_packet_size as usize) {
                return Err(MqttBrokerError::PacketLengthError(publish.payload.len()));
            }
        }

        retry_times += 1;
        let mut publish = publish.clone();
        publish.dup = retry_times >= 2;

        let mut contain_properties = false;
        if let Some(protocol) = connection_manager.get_connect_protocol(connect_id) {
            if MqttProtocol::is_mqtt5(&protocol) {
                contain_properties = true;
            }
        }

        let resp = if contain_properties {
            ResponsePackage {
                connection_id: connect_id,
                packet: MqttPacket::Publish(publish.clone(), publish_properties.clone()),
            }
        } else {
            ResponsePackage {
                connection_id: connect_id,
                packet: MqttPacket::Publish(publish.clone(), None),
            }
        };

        select! {
            val = stop_rx.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return Ok(());
                    }
                }
            }
            val = publish_message_to_client(
                resp.clone(),
                connection_manager,
            ) =>{
                match val{
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        error!(
                            "Failed to write QOS2 Publish message to response queue, failure message: {}",
                            e.to_string()
                        );
                        sleep(Duration::from_millis(1)).await;
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn qos2_send_pubrel(
    metadata_cache: &Arc<CacheManager>,
    client_id: &str,
    pkid: u16,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();

    loop {
        let connect_id = if let Some(id) = metadata_cache.get_connect_id(client_id) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        let pubrel = PubRel {
            pkid,
            reason: Some(protocol::mqtt::common::PubRelReason::Success),
        };

        let pubrel_resp = ResponsePackage {
            connection_id: connect_id,
            packet: MqttPacket::PubRel(pubrel, None),
        };

        select! {
            val = stop_rx.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return;
                    }
                }
            }

            val = publish_message_to_client(
                pubrel_resp.clone(),
                connection_manager,
            ) =>{
                match val{
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        error!(
                            "Failed to write PubRel message to response queue, failure message: {}",
                            e.to_string()
                        );
                    }
                }
            }

        }
    }
}

pub async fn loop_commit_offset<S>(
    message_storage: &MessageStorage<S>,
    topic_id: &str,
    group_id: &str,
    offset: u128,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    loop {
        match message_storage
            .commit_group_offset(topic_id.to_owned(), group_id.to_owned(), offset)
            .await
        {
            Ok(_) => {
                break;
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}

// When the subscription QOS is 0,
// the message can be pushed directly to the request return queue without the need for a retry mechanism.
pub async fn publish_message_qos0(
    metadata_cache: &Arc<CacheManager>,
    mqtt_client_id: &str,
    publish: &Publish,
    properties: &Option<PublishProperties>,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let connect_id;
    loop {
        if let Ok(flag) = stop_sx.subscribe().try_recv() {
            if flag {
                return;
            }
        }

        if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id) {
            connect_id = id;
            break;
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };
    }

    if let Some(conn) = metadata_cache.get_connection(connect_id) {
        if publish.payload.len() > (conn.max_packet_size as usize) {
            return;
        }
    }

    let mut contain_properties = false;
    if let Some(protocol) = connection_manager.get_connect_protocol(connect_id) {
        if MqttProtocol::is_mqtt5(&protocol) {
            contain_properties = true;
        }
    }

    let resp = if contain_properties {
        ResponsePackage {
            connection_id: connect_id,
            packet: MqttPacket::Publish(publish.clone(), properties.clone()),
        }
    } else {
        ResponsePackage {
            connection_id: connect_id,
            packet: MqttPacket::Publish(publish.clone(), None),
        }
    };

    // 2. publish to mqtt client
    match publish_message_to_client(resp.clone(), connection_manager).await {
        Ok(_) => {}
        Err(e) => {
            error!(
                "Failed Publish message to response queue, failure message: {}",
                e.to_string()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::tools::unique_id;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::topic::MqttTopic;
    use protocol::mqtt::common::QoS;

    use crate::handler::cache::CacheManager;
    use crate::subscribe::sub_common::{
        decode_share_info, get_sub_topic_id_list, is_share_sub, min_qos, path_regex_match,
        sub_path_validator,
    };

    #[tokio::test]
    async fn is_share_sub_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        assert!(is_share_sub(sub1));
        assert!(is_share_sub(sub2));
        assert!(is_share_sub(sub3));
        assert!(is_share_sub(sub4));

        let sub5 = "/comsumer1/$share/finance/#".to_string();
        let sub6 = "/comsumer1/$share/finance/$share".to_string();

        assert!(!is_share_sub(sub5));
        assert!(!is_share_sub(sub6));
    }

    #[tokio::test]
    #[ignore]
    async fn decode_share_info_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        let (group_name, topic_name) = decode_share_info(sub1);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(sub2);
        assert_eq!(group_name, "consumer2".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(sub3);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/#".to_string());

        let (group_name, topic_name) = decode_share_info(sub4);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/finance/#".to_string());
    }
    #[test]
    fn path_regex_match_test() {
        let topic_name = "/loboxu/test".to_string();
        let sub_regex = "/loboxu/#".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = "/topic/test".to_string();
        let sub_regex = "/topic/test".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/1/temperature".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/1/2/temperature3".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert!(!path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert!(!path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"/sensor/+".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/temperature3/tmpq".to_string();
        let sub_regex = r"/sensor/#".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = "/topic/test".to_string();
        let sub_regex = "$share/groupname/topic/test".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/1/temperature".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/1/2/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert!(!path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert!(!path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/temperature3/tmpq".to_string();
        let sub_regex = r"$share/groupname/sensor/#".to_string();
        assert!(path_regex_match(topic_name, sub_regex));
    }

    #[test]
    fn max_qos_test() {
        let mut sub_max_qos = QoS::AtMostOnce;
        let mut msg_qos = QoS::AtLeastOnce;
        assert_eq!(min_qos(msg_qos, sub_max_qos), sub_max_qos);

        msg_qos = QoS::AtMostOnce;
        sub_max_qos = QoS::AtLeastOnce;
        assert_eq!(min_qos(msg_qos, sub_max_qos), msg_qos);
    }

    #[tokio::test]
    async fn get_sub_topic_list_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(100));
        let metadata_cache = Arc::new(CacheManager::new(client_pool, "test-cluster".to_string()));
        let topic_name = "/test/topic".to_string();
        let topic = MqttTopic::new(unique_id(), "c1".to_string(), topic_name.clone());
        metadata_cache.add_topic(&topic_name, &topic);

        let sub_path = "/test/topic".to_string();
        let result = get_sub_topic_id_list(metadata_cache.clone(), sub_path).await;
        assert!(result.len() == 1);
        assert_eq!(result.first().unwrap().clone(), topic.topic_id);
    }

    #[tokio::test]
    async fn path_validator_test() {
        let path = "/loboxu/test".to_string();
        assert!(sub_path_validator(path));

        let path = "/loboxu/#".to_string();
        assert!(sub_path_validator(path));

        let path = "/loboxu/+".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/#".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/#/test".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/+/test".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/+test".to_string();
        assert!(!sub_path_validator(path));

        let path = "$share/loboxu/#test".to_string();
        assert!(!sub_path_validator(path));

        let path = "$share/loboxu/*test".to_string();
        assert!(!sub_path_validator(path));
    }
}
