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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::message::MessageStorage;

use common_base::error::common::CommonError;
use common_base::error::not_record_error;
use common_base::utils::topic_util::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub};
use common_config::broker::broker_config;
use grpc_clients::meta::mqtt::call::placement_get_share_sub_leader;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::subscribe_data::{is_mqtt_queue_sub, is_mqtt_share_sub};
use protocol::meta::meta_service_mqtt::{GetShareSubLeaderReply, GetShareSubLeaderRequest};
use protocol::mqtt::common::{
    Filter, MqttProtocol, RetainHandling, SubAck, SubscribeProperties, SubscribeReasonCode,
};
use protocol::mqtt::common::{MqttPacket, QoS};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SUBSCRIBE_WILDCARDS_1: &str = "+";
const SUBSCRIBE_WILDCARDS_2: &str = "#";
const SUBSCRIBE_SPLIT_DELIMITER: &str = "/";
const SUBSCRIBE_NAME_REGEX: &str = r"^[\$a-zA-Z0-9_#+/]+$";
pub const SHARE_QUEUE_DEFAULT_GROUP_NAME: &str = "$queue_group_robustmq";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub protocol: MqttProtocol,
    pub client_id: String,
    pub sub_path: String,
    pub rewrite_sub_path: Option<String>,
    pub topic_name: String,
    pub group_name: Option<String>,
    pub topic_id: String,
    pub qos: QoS,
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub retain_forward_rule: RetainHandling,
    pub subscription_identifier: Option<usize>,
    pub create_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscribeData {
    pub protocol: MqttProtocol,
    pub filter: Filter,
    pub subscribe_properties: Option<SubscribeProperties>,
}

#[derive(Clone, Debug)]
pub struct SubPublishParam {
    pub subscribe: Subscriber,
    pub packet: MqttPacket,
    pub create_time: u128,
    pub pkid: u16,
    pub group_id: String,
}

impl SubPublishParam {
    pub fn new(
        subscribe: Subscriber,
        packet: MqttPacket,
        create_time: u128,
        group_id: String,
        pkid: u16,
    ) -> Self {
        SubPublishParam {
            subscribe,
            packet,
            create_time,
            pkid,
            group_id,
        }
    }
}

pub fn is_ignore_push_error(e: &MqttBrokerError) -> bool {
    if not_record_error(&e.to_string()) {
        return true;
    }

    match e {
        MqttBrokerError::SessionNullSkipPushMessage(_) => {}
        MqttBrokerError::ConnectionNullSkipPushMessage(_) => {}
        MqttBrokerError::NotObtainAvailableConnection(_, _) => {}
        MqttBrokerError::OperationTimeout(_, _) => {}
        _ => {
            return false;
        }
    }

    true
}

pub fn sub_path_validator(sub_path: &str) -> ResultMqttBrokerError {
    let regex = Regex::new(SUBSCRIBE_NAME_REGEX)?;

    if !regex.is_match(sub_path) {
        return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
    }

    for path in sub_path.split(SUBSCRIBE_SPLIT_DELIMITER) {
        if path.contains(SUBSCRIBE_WILDCARDS_1) && path != SUBSCRIBE_WILDCARDS_1 {
            return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
        }
        if path.contains(SUBSCRIBE_WILDCARDS_2) && path != SUBSCRIBE_WILDCARDS_2 {
            return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
        }
    }

    Ok(())
}

pub fn is_wildcards(sub_path: &str) -> bool {
    sub_path.contains(SUBSCRIBE_WILDCARDS_1) || sub_path.contains(SUBSCRIBE_WILDCARDS_2)
}

pub fn is_match_sub_and_topic(sub_path: &str, topic: &str) -> ResultMqttBrokerError {
    let path = decode_sub_path(sub_path);
    let topic_name = decode_sub_path(topic);

    if *path == topic_name {
        return Ok(());
    }

    if is_wildcards(&path) {
        if let Ok(regex) = build_sub_path_regex(&path) {
            if regex.is_match(&topic_name) {
                return Ok(());
            }
        }
    }

    Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()))
}

pub fn build_sub_path_regex(sub_path: &str) -> Result<Regex, MqttBrokerError> {
    let path = decode_sub_path(sub_path);

    if !is_wildcards(&path) {
        return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
    }

    // +
    if path.contains("+") {
        let mut sub_regex = path.replace("+", "[^+*/]+");
        if path.contains("#") {
            if path.split("/").last().unwrap() != "#" {
                return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
            }
            sub_regex = sub_regex.replace("#", "[^+#]+");
        }
        return Ok(Regex::new(&sub_regex.to_string())?);
    }

    // #
    if path.split("/").last().unwrap() != "#" {
        return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
    }
    let sub_regex = path.replace("#", "[^+#]+");
    Ok(Regex::new(&sub_regex.to_string())?)
}

pub fn decode_sub_path(sub_path: &str) -> String {
    if is_mqtt_share_sub(sub_path) {
        let (_, group_path) = decode_share_info(sub_path);
        group_path
    } else if is_mqtt_queue_sub(sub_path) {
        decode_queue_info(sub_path)
    } else if is_exclusive_sub(sub_path) {
        decode_exclusive_sub_path_to_topic_name(sub_path).to_owned()
    } else {
        sub_path.to_owned()
    }
}

pub fn min_qos(qos: QoS, sub_qos: QoS) -> QoS {
    if qos <= sub_qos {
        return qos;
    }
    sub_qos
}

pub async fn get_sub_topic_id_list(
    metadata_cache: &Arc<MQTTCacheManager>,
    sub_path: &str,
) -> Vec<String> {
    let mut result = Vec::new();
    if is_wildcards(sub_path) {
        if let Ok(regex) = build_sub_path_regex(sub_path) {
            for (topic_id, topic_name) in metadata_cache.topic_id_name.clone() {
                if regex.is_match(&topic_name) {
                    result.push(topic_id);
                }
            }
        };
    } else {
        for (topic_id, topic_name) in metadata_cache.topic_id_name.clone() {
            if topic_name == *sub_path {
                result.push(topic_id);
            }
        }
    }

    result
}

pub fn decode_share_group_and_path(path: &str) -> (String, String) {
    if is_mqtt_queue_sub(path) {
        (
            SHARE_QUEUE_DEFAULT_GROUP_NAME.to_string(),
            decode_queue_info(path),
        )
    } else {
        decode_share_info(path)
    }
}

fn decode_share_info(sub_name: &str) -> (String, String) {
    let mut str_slice: Vec<&str> = sub_name.split("/").collect();
    str_slice.remove(0);
    let group_name = str_slice.remove(0).to_string();
    let sub_name = format!("/{}", str_slice.join("/"));
    (group_name, sub_name)
}

fn decode_queue_info(sub_name: &str) -> String {
    let mut str_slice: Vec<&str> = sub_name.split("/").collect();
    str_slice.remove(0);
    format!("/{}", str_slice.join("/"))
}

pub async fn get_share_sub_leader(
    client_pool: &Arc<ClientPool>,
    group_name: &String,
) -> Result<GetShareSubLeaderReply, CommonError> {
    let conf = broker_config();
    let req = GetShareSubLeaderRequest {
        cluster_name: conf.cluster_name.to_owned(),
        group_name: group_name.to_owned(),
    };
    placement_get_share_sub_leader(client_pool, &conf.get_meta_service_addr(), req).await
}

pub async fn loop_commit_offset(
    message_storage: &MessageStorage,
    topic_id: &str,
    group_id: &str,
    offset: u64,
) -> ResultMqttBrokerError {
    message_storage
        .commit_group_offset(group_id, topic_id, offset)
        .await?;
    Ok(())
}

pub fn is_error_by_suback(suback: &SubAck) -> bool {
    for reason in suback.return_codes.clone() {
        if !(reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtLeastOnce)
            || reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtMostOnce)
            || reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::ExactlyOnce)
            || reason == SubscribeReasonCode::QoS0
            || reason == SubscribeReasonCode::QoS1
            || reason == SubscribeReasonCode::QoS2)
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::subscribe::common::{
        build_sub_path_regex, decode_queue_info, decode_share_info, decode_sub_path,
        get_sub_topic_id_list, is_match_sub_and_topic, is_wildcards, min_qos, sub_path_validator,
    };
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::subscribe_data::{is_mqtt_queue_sub, is_mqtt_share_sub};
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::QoS;

    #[tokio::test]
    async fn is_wildcards_test() {
        assert!(!is_wildcards("/test/t1"));
        assert!(is_wildcards("/test/+"));
        assert!(is_wildcards("/test/#"));
    }

    #[tokio::test]
    async fn decode_queue_info_test() {
        let res = decode_queue_info("$queue/vvv/v1");
        println!("{res}");
    }

    #[tokio::test]
    async fn is_match_sub_and_topic_test() {
        let topic_name = "/loboxu/test";
        let sub_regex = "/loboxu/#";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = "/topic/test";
        let sub_regex = "/topic/test";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = r"/sensor/1/temperature";
        let sub_regex = r"/sensor/+/temperature";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = r"/sensor/1/2/temperature3";
        let sub_regex = r"/sensor/+/temperature";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_err());

        let topic_name = r"/sensor/temperature3";
        let sub_regex = r"/sensor/+/temperature";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_err());

        let topic_name = r"/sensor/temperature3";
        let sub_regex = r"/sensor/+";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = r"/sensor/temperature3/tmpq";
        let sub_regex = r"/sensor/#";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = "/topic/test";
        let sub_regex = "$share/groupname/topic/test";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = r"/sensor/1/temperature";
        let sub_regex = r"$share/groupname/sensor/+/temperature";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = r"/sensor/1/2/temperature3";
        let sub_regex = r"$share/groupname/sensor/+/temperature";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_err());

        let topic_name = r"/sensor/temperature3";
        let sub_regex = r"$share/groupname/sensor/+/temperature";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_err());

        let topic_name = r"/sensor/temperature3";
        let sub_regex = r"$share/groupname/sensor/+";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = r"/sensor/temperature3/tmpq";
        let sub_regex = r"$share/groupname/sensor/#";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());

        let topic_name = r"y/a/z/b";
        let sub_regex = r"y/+/z/#";
        assert!(is_match_sub_and_topic(sub_regex, topic_name).is_ok());
    }

    #[tokio::test]
    async fn build_sub_path_regex_test() {
        let topic_name = "/loboxu/test";
        let sub_regex = "/loboxu/#";
        let regex = build_sub_path_regex(sub_regex).unwrap();
        assert!(regex.is_match(topic_name));

        let topic_name = r"y/a/z/b";
        let sub_regex = r"y/+/z/#";
        let regex = build_sub_path_regex(sub_regex).unwrap();
        assert!(regex.is_match(topic_name));

        let topic_name = r"/sensor/1/temperature";
        let sub_regex = r"$share/groupname/sensor/+/temperature";
        let regex = build_sub_path_regex(sub_regex).unwrap();
        assert!(regex.is_match(topic_name));

        let topic_name = r"/sensor/1/2/temperature3";
        let sub_regex = r"$share/groupname/sensor/+/temperature";
        let regex = build_sub_path_regex(sub_regex).unwrap();
        assert!(!regex.is_match(topic_name));

        let topic_name = r"/sensor/temperature3";
        let sub_regex = r"$share/groupname/sensor/+/temperature";
        let regex = build_sub_path_regex(sub_regex).unwrap();
        assert!(!regex.is_match(topic_name));

        let topic_name = r"/sensor/temperature3";
        let sub_regex = r"$share/groupname/sensor/+";
        let regex = build_sub_path_regex(sub_regex).unwrap();
        assert!(regex.is_match(topic_name));

        let topic_name = r"/sensor/temperature3/tmpq";
        let sub_regex = r"$share/groupname/sensor/#";
        let regex = build_sub_path_regex(sub_regex).unwrap();
        assert!(regex.is_match(topic_name));
    }

    #[tokio::test]
    async fn is_share_sub_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        assert!(is_mqtt_share_sub(&sub1));
        assert!(is_mqtt_share_sub(&sub2));
        assert!(is_mqtt_share_sub(&sub3));
        assert!(is_mqtt_share_sub(&sub4));

        let sub5 = "/comsumer1/$share/finance/#".to_string();
        let sub6 = "/comsumer1/$share/finance/$share".to_string();

        assert!(!is_mqtt_share_sub(&sub5));
        assert!(!is_mqtt_share_sub(&sub6));
    }

    #[tokio::test]
    async fn is_queue_sub_test() {
        assert!(is_mqtt_queue_sub("$queue/vvv/v1"));
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
        let metadata_cache = test_build_mqtt_cache_manager();
        let topic_name = "/test/topic".to_string();
        let topic = MQTTTopic::new(unique_id(), "c1".to_string(), topic_name.clone());
        metadata_cache.add_topic(&topic_name, &topic);

        let sub_path = "/test/topic".to_string();
        let result = get_sub_topic_id_list(&metadata_cache, &sub_path).await;
        assert!(result.len() == 1);
        assert_eq!(result.first().unwrap().clone(), topic.topic_id);
    }

    #[tokio::test]
    async fn sub_path_validator_test() {
        let path = "/loboxu/test";
        assert!(sub_path_validator(path).is_ok());

        let path = "/loboxu/#";
        assert!(sub_path_validator(path).is_ok());

        let path = "/loboxu/+";
        assert!(sub_path_validator(path).is_ok());

        let path = "$share/loboxu/#";
        assert!(sub_path_validator(path).is_ok());

        let path = "$share/loboxu/#/test";
        assert!(sub_path_validator(path).is_ok());

        let path = "$share/loboxu/+/test";
        assert!(sub_path_validator(path).is_ok());

        let path = "$share/loboxu/+test";
        assert!(sub_path_validator(path).is_err());

        let path = "$share/loboxu/#test";
        assert!(sub_path_validator(path).is_err());

        let path = "$share/loboxu/*test";
        assert!(sub_path_validator(path).is_err());
    }

    #[tokio::test]
    async fn decode_sub_path_sub_test() {
        let path = "$share/group1/topic1/1".to_string();
        assert_eq!(decode_sub_path(&path), "/topic1/1".to_string());

        let path = "$queue/topic1/1".to_string();
        assert_eq!(decode_sub_path(&path), "/topic1/1".to_string());

        let path = "/topic1/1".to_string();
        assert_eq!(decode_sub_path(&path), "/topic1/1".to_string());

        let path = "$exclusive/topic1/1".to_string();
        assert_eq!(decode_sub_path(&path), "/topic1/1".to_string());
    }
}
