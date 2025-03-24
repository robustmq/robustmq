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
use common_base::tools::now_mills;
use grpc_clients::placement::mqtt::call::placement_get_share_sub_leader;
use grpc_clients::pool::ClientPool;
use log::{error, warn};
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{MqttPacket, MqttProtocol, PubRel, QoS};
use protocol::placement_center::placement_center_mqtt::{
    GetShareSubLeaderReply, GetShareSubLeaderRequest,
};
use regex::Regex;
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::{sleep, timeout};

use super::subscriber::SubPublishParam;
use crate::handler::cache::{CacheManager, QosAckPackageData, QosAckPackageType};
use crate::handler::error::MqttBrokerError;
use crate::observability::slow::sub::{record_slow_sub_data, SlowSubData};
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::ResponsePackage;
use crate::storage::message::MessageStorage;

const SHARE_SUB_PREFIX: &str = "$share";

const QUEUE_SUB_PREFIX: &str = "$queue";

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

pub fn path_regex_match(topic_name: &str, sub_path: &str) -> bool {
    let path = if is_share_sub(sub_path) {
        let (_, group_path) = decode_share_info(sub_path);
        group_path
    } else if is_queue_sub(sub_path) {
        decode_queue_info(sub_path)
    } else {
        sub_path.to_owned()
    };

    let topic = if is_share_sub(topic_name) {
        let (_, group_path) = decode_share_info(topic_name);
        group_path
    } else if is_queue_sub(topic_name) {
        decode_queue_info(topic_name)
    } else {
        topic_name.to_owned()
    };

    // Path perfect matching
    if topic == path {
        return true;
    }

    if path.contains("+") {
        let sub_regex = path.replace("+", "[^+*/]+");
        let re = Regex::new(&sub_regex.to_string()).unwrap();
        return re.is_match(&topic);
    }

    if path.contains("#") {
        if path.split("/").last().unwrap() != "#" {
            return false;
        }
        let sub_regex = path.replace("#", "[^+#]+");
        let re = Regex::new(&sub_regex.to_string()).unwrap();
        return re.is_match(&topic);
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
    metadata_cache: &Arc<CacheManager>,
    sub_path: &str,
) -> Vec<String> {
    let mut result = Vec::new();
    for (topic_id, topic_name) in metadata_cache.topic_id_name.clone() {
        if path_regex_match(&topic_name, sub_path) {
            result.push(topic_id);
        }
    }
    result
}

pub fn is_share_sub(sub_name: &str) -> bool {
    sub_name.starts_with(SHARE_SUB_PREFIX)
}

pub fn is_queue_sub(sub_name: &str) -> bool {
    sub_name.starts_with(QUEUE_SUB_PREFIX)
}

pub fn decode_share_info(sub_name: &str) -> (String, String) {
    let mut str_slice: Vec<&str> = sub_name.split("/").collect();
    str_slice.remove(0);
    let group_name = str_slice.remove(0).to_string();
    let sub_name = format!("/{}", str_slice.join("/"));
    (group_name, sub_name)
}

pub fn decode_queue_info(sub_name: &str) -> String {
    let mut str_slice: Vec<&str> = sub_name.split("/").collect();
    str_slice.remove(0);
    format!("/{}", str_slice.join("/"))
}

pub async fn get_share_sub_leader(
    client_pool: &Arc<ClientPool>,
    group_name: &String,
) -> Result<GetShareSubLeaderReply, CommonError> {
    let conf = broker_mqtt_conf();
    let req = GetShareSubLeaderRequest {
        cluster_name: conf.cluster_name.to_owned(),
        group_name: group_name.to_owned(),
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
    sub_pub_param: &SubPublishParam,
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<CacheManager>,
) -> Result<(), MqttBrokerError> {
    if let Some(protocol) = connection_manager.get_connect_protocol(resp.connection_id) {
        let response: MqttPacketWrapper = MqttPacketWrapper {
            protocol_version: protocol.clone().into(),
            packet: resp.packet,
        };

        if connection_manager.is_websocket(resp.connection_id) {
            let mut codec = MqttCodec::new(Some(protocol.into()));
            let mut buff = BytesMut::new();
            match codec.encode_data(response.clone(), &mut buff) {
                Ok(()) => {}
                Err(e) => {
                    error!("Websocket encode back packet failed with error message: {e:?}");
                }
            }
            connection_manager
                .write_websocket_frame(resp.connection_id, response, Message::Binary(buff.to_vec()))
                .await?;
        } else {
            connection_manager
                .write_tcp_frame(resp.connection_id, response)
                .await?
        }

        // record slow sub data
        if metadata_cache.get_slow_sub_config().enable && sub_pub_param.create_time > 0 {
            let slow_data = SlowSubData::build(
                sub_pub_param.subscribe.sub_path.clone(),
                sub_pub_param.subscribe.client_id.clone(),
                sub_pub_param.subscribe.topic_name.clone(),
                (now_mills() - sub_pub_param.create_time) as u64,
            );
            record_slow_sub_data(slow_data, metadata_cache.get_slow_sub_config().whole_ms)?;
        }
    }

    Ok(())
}

pub async fn wait_pub_ack(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) {
    let wait_pub_rec_fn = async || -> Result<(), MqttBrokerError> {
        match timeout(Duration::from_secs(30), wait_packet_ack(wait_ack_sx)).await {
            Ok(Some(data)) => {
                if data.ack_type == QosAckPackageType::PubAck && data.pkid == sub_pub_param.pkid {
                    return Ok(());
                }
            }
            Ok(None) => {}
            Err(e) => {
                publish_message_qos(metadata_cache, connection_manager, sub_pub_param, stop_sx)
                    .await;
                return Err(MqttBrokerError::CommonError(
                    format!(
                        "Push QOS1 Publish message to client {}, wait PubAck timeout, more than 30s, error message :{:?}",
                        sub_pub_param.subscribe.client_id,
                        e
                    )
                ));
            }
        };

        Err(MqttBrokerError::CommonError(
            "Sending a Qos1 message to the client did not receive a correct PubAck return"
                .to_owned(),
        ))
    };

    let mut stop_recv = stop_sx.subscribe();
    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return;
                    }
                }
            }
            val = wait_pub_rec_fn() => {
                if let Err(e) = val {
                    error!("{:?}",e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                break;
            }
        }
    }
}

pub async fn wait_pub_rec(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) {
    let wait_pub_rec_fn = async || -> Result<(), MqttBrokerError> {
        match timeout(Duration::from_secs(30), wait_packet_ack(wait_ack_sx)).await {
            Ok(Some(data)) => {
                if data.ack_type == QosAckPackageType::PubRec && data.pkid == sub_pub_param.pkid {
                    return Ok(());
                }
            }
            Ok(None) => {}
            Err(e) => {
                publish_message_qos(metadata_cache, connection_manager, sub_pub_param, stop_sx)
                    .await;
                return Err(MqttBrokerError::CommonError(
                    format!(
                        "Push QOS2 Publish message to client {}, wait pubrec timeout, more than 30s, error message :{:?}",
                        sub_pub_param.subscribe.client_id,
                        e
                    )
                ));
            }
        };

        Err(MqttBrokerError::CommonError(
            "Sending a Qos 2 message to the client did not receive a correct PubRec return"
                .to_owned(),
        ))
    };

    let mut stop_recv = stop_sx.subscribe();
    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return;
                    }
                }
            }
            val = wait_pub_rec_fn() => {
                if let Err(e) = val {
                    error!("{:?}",e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                break;
            }
        }
    }
}

pub async fn wait_pub_comp(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) {
    let wait_pub_rec_fn = async || -> Result<(), MqttBrokerError> {
        match timeout(Duration::from_secs(30), wait_packet_ack(wait_ack_sx)).await {
            Ok(Some(data)) => {
                if data.ack_type == QosAckPackageType::PubComp && data.pkid == sub_pub_param.pkid {
                    return Ok(());
                }
            }
            Ok(None) => {}
            Err(e) => {
                qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx).await;
                return Err(MqttBrokerError::CommonError(
                    format!(
                        "Push QOS2 Publish message to client {}, wait PubComp timeout, more than 30s, error message :{:?}",
                        sub_pub_param.subscribe.client_id,
                        e
                    )
                ));
            }
        };

        Err(MqttBrokerError::CommonError(
            "Sending a Qos 2 message to the client did not receive a correct PubComp return"
                .to_owned(),
        ))
    };

    let mut stop_recv = stop_sx.subscribe();
    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return;
                    }
                }
            }
            val = wait_pub_rec_fn() => {
                if let Err(e) = val {
                    error!("{:?}",e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                break;
            }
        }
    }
}

pub async fn qos2_send_pubrel(
    metadata_cache: &Arc<CacheManager>,
    sub_pub_param: &SubPublishParam,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();

    loop {
        let connect_id =
            if let Some(id) = metadata_cache.get_connect_id(&sub_pub_param.subscribe.client_id) {
                id
            } else {
                sleep(Duration::from_secs(1)).await;
                continue;
            };

        let pubrel = PubRel {
            pkid: sub_pub_param.pkid,
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
                sub_pub_param,
                connection_manager,
                metadata_cache
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
    offset: u64,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    loop {
        match message_storage
            .commit_group_offset(group_id, topic_id, offset)
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
pub async fn publish_message_qos(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
) {
    let push_to_connect = async |client_id: String| -> Result<(), MqttBrokerError> {
        if metadata_cache.get_session_info(&client_id).is_none() {
            warn!("Client {} is not online, skip push message", client_id);
            return Ok(());
        }

        let connect_id_op = metadata_cache.get_connect_id(&client_id);
        if connect_id_op.is_none() {
            return Err(MqttBrokerError::ClientNoAvailableCOnnection(client_id));
        }

        let connect_id = connect_id_op.unwrap();

        if let Some(conn) = metadata_cache.get_connection(connect_id) {
            if sub_pub_param.publish.payload.len() > (conn.max_packet_size as usize) {
                return Ok(());
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
                packet: MqttPacket::Publish(
                    sub_pub_param.publish.clone(),
                    sub_pub_param.properties.clone(),
                ),
            }
        } else {
            ResponsePackage {
                connection_id: connect_id,
                packet: MqttPacket::Publish(sub_pub_param.publish.clone(), None),
            }
        };

        // 2. publish to mqtt client
        publish_message_to_client(
            resp.clone(),
            sub_pub_param,
            connection_manager,
            metadata_cache,
        )
        .await?;
        Ok(())
    };

    let mut stop_recv = stop_sx.subscribe();
    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return;
                    }
                }
            }
            val = push_to_connect(sub_pub_param.subscribe.client_id.clone()) => {
                if let Err(e) = val{
                    error!(
                        "Push Qos message to client {} failed, error message :{:?}",
                        sub_pub_param.subscribe.client_id, e
                    );
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                break;
            }

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

        assert!(is_share_sub(&sub1));
        assert!(is_share_sub(&sub2));
        assert!(is_share_sub(&sub3));
        assert!(is_share_sub(&sub4));

        let sub5 = "/comsumer1/$share/finance/#".to_string();
        let sub6 = "/comsumer1/$share/finance/$share".to_string();

        assert!(!is_share_sub(&sub5));
        assert!(!is_share_sub(&sub6));
    }

    #[tokio::test]
    #[ignore]
    async fn decode_share_info_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        let (group_name, topic_name) = decode_share_info(&sub1);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(&sub2);
        assert_eq!(group_name, "consumer2".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(&sub3);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/#".to_string());

        let (group_name, topic_name) = decode_share_info(&sub4);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/finance/#".to_string());
    }
    #[test]
    fn path_regex_match_test() {
        let topic_name = "/loboxu/test".to_string();
        let sub_regex = "/loboxu/#".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = "/topic/test".to_string();
        let sub_regex = "/topic/test".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/1/temperature".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/1/2/temperature3".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert!(!path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert!(!path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"/sensor/+".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/temperature3/tmpq".to_string();
        let sub_regex = r"/sensor/#".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = "/topic/test".to_string();
        let sub_regex = "$share/groupname/topic/test".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/1/temperature".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/1/2/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert!(!path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert!(!path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));

        let topic_name = r"/sensor/temperature3/tmpq".to_string();
        let sub_regex = r"$share/groupname/sensor/#".to_string();
        assert!(path_regex_match(&topic_name, &sub_regex));
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
        let result = get_sub_topic_id_list(&metadata_cache, &sub_path).await;
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
