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
use crate::handler::sub_exclusive::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub};
use crate::handler::sub_share::{decode_share_info, is_mqtt_share_subscribe};
use crate::handler::sub_wildcards::is_wildcards;
use common_base::error::not_record_error;
use protocol::mqtt::common::{
    Filter, MqttProtocol, RetainHandling, SubAck, SubscribeProperties, SubscribeReasonCode,
};
use protocol::mqtt::common::{MqttPacket, QoS};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub client_id: String,
    pub sub_path: String,
    pub rewrite_sub_path: Option<String>,
    pub topic_name: String,
    pub group_name: String,
    pub protocol: MqttProtocol,
    pub qos: QoS,
    pub no_local: bool,
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
    pub packet: MqttPacket,
    pub create_time: u64,
    pub client_id: String,
    pub p_kid: u16,
    pub qos: QoS,
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
    if is_mqtt_share_subscribe(sub_path) {
        let (_, group_path) = decode_share_info(sub_path);
        group_path
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

pub async fn get_sub_topic_name_list(
    metadata_cache: &Arc<MQTTCacheManager>,
    sub_path: &str,
) -> Vec<String> {
    let mut result = Vec::new();
    if is_wildcards(sub_path) {
        if let Ok(regex) = build_sub_path_regex(sub_path) {
            for (_, topic) in metadata_cache.topic_info.clone() {
                if regex.is_match(&topic.topic_name) {
                    result.push(topic.topic_name);
                }
            }
        };
    } else {
        for (_, topic) in metadata_cache.topic_info.clone() {
            if topic.topic_name == *sub_path {
                result.push(topic.topic_name);
            }
        }
    }

    result
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
        build_sub_path_regex, decode_share_info, decode_sub_path, get_sub_topic_name_list,
        is_match_sub_and_topic, is_wildcards, min_qos,
    };
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::QoS;

    #[tokio::test]
    async fn is_wildcards_test() {
        assert!(!is_wildcards("/test/t1"));
        assert!(is_wildcards("/test/+"));
        assert!(is_wildcards("/test/#"));
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
        let metadata_cache = test_build_mqtt_cache_manager().await;
        let topic_name = "/test/topic".to_string();
        let topic = MQTTTopic::new(topic_name.clone());
        metadata_cache.add_topic(&topic_name, &topic);

        let sub_path = "/test/topic".to_string();
        let result = get_sub_topic_name_list(&metadata_cache, &sub_path).await;
        assert!(result.len() == 1);
        assert_eq!(result.first().unwrap().clone(), topic.topic_name);
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
