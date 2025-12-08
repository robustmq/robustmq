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

use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::handler::sub_exclusive::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub};
use crate::handler::sub_share::{decode_share_info, is_mqtt_share_subscribe};
use crate::handler::sub_wildcards::is_wildcards;
use crate::handler::tool::ResultMqttBrokerError;
use common_base::error::not_record_error;
use common_metrics::mqtt::subscribe::{
    record_subscribe_bytes_sent, record_subscribe_messages_sent, record_subscribe_topic_bytes_sent,
    record_subscribe_topic_messages_sent,
};
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

    matches!(
        e,
        MqttBrokerError::SessionNullSkipPushMessage(_)
            | MqttBrokerError::ConnectionNullSkipPushMessage(_)
            | MqttBrokerError::NotObtainAvailableConnection(_, _)
            | MqttBrokerError::OperationTimeout(_, _)
    )
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

    // Handle + wildcard (single level)
    if path.contains("+") {
        let mut sub_regex = path.replace("+", "[^/]+");
        if path.contains("#") {
            // Validate # is at the end
            if !path.ends_with("#") || path.split("/").last() != Some("#") {
                return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
            }
            sub_regex = sub_regex.replace("#", ".*");
        }
        return Ok(Regex::new(&format!("^{}$", sub_regex))?);
    }

    // Handle # wildcard (multi level)
    if !path.ends_with("#") || path.split("/").last() != Some("#") {
        return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
    }
    let sub_regex = path.replace("#", ".*");
    Ok(Regex::new(&format!("^{}$", sub_regex))?)
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
            for row in metadata_cache.topic_info.iter() {
                if regex.is_match(&row.value().topic_name) {
                    result.push(row.value().topic_name.clone());
                }
            }
        }
    } else {
        for row in metadata_cache.topic_info.iter() {
            if row.value().topic_name == *sub_path {
                result.push(row.value().topic_name.clone());
                break;
            }
        }
    }

    result
}

pub fn is_error_by_suback(suback: &SubAck) -> bool {
    suback.return_codes.iter().any(|reason| {
        !matches!(
            reason,
            SubscribeReasonCode::Success(QoS::AtLeastOnce)
                | SubscribeReasonCode::Success(QoS::AtMostOnce)
                | SubscribeReasonCode::Success(QoS::ExactlyOnce)
                | SubscribeReasonCode::QoS0
                | SubscribeReasonCode::QoS1
                | SubscribeReasonCode::QoS2
        )
    })
}

pub fn record_sub_send_metrics(
    client_id: &str,
    path: &str,
    topic_name: &str,
    data_size: u64,
    success: bool,
) {
    record_subscribe_bytes_sent(client_id, path, data_size, success);
    record_subscribe_topic_bytes_sent(client_id, path, topic_name, data_size, success);

    record_subscribe_messages_sent(client_id, path, success);
    record_subscribe_topic_messages_sent(client_id, path, topic_name, success);
}

#[cfg(test)]
mod tests {
    use crate::handler::error::MqttBrokerError;
    use crate::handler::tool::test_build_mqtt_cache_manager;
    use crate::subscribe::common::{
        build_sub_path_regex, decode_share_info, decode_sub_path, get_sub_topic_name_list,
        is_error_by_suback, is_ignore_push_error, is_match_sub_and_topic, is_wildcards, min_qos,
    };
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::{QoS, SubAck, SubscribeReasonCode};

    #[tokio::test]
    async fn is_wildcards_test() {
        assert!(!is_wildcards("/test/t1"));
        assert!(is_wildcards("/test/+"));
        assert!(is_wildcards("/test/#"));
    }

    #[tokio::test]
    async fn is_match_sub_and_topic_test() {
        let test_cases = vec![
            // (sub_pattern, topic, should_match)
            ("/loboxu/#", "/loboxu/test", true),
            ("/topic/test", "/topic/test", true),
            ("/sensor/+/temperature", "/sensor/1/temperature", true),
            ("/sensor/+/temperature", "/sensor/1/2/temperature3", false),
            ("/sensor/+/temperature", "/sensor/temperature3", false),
            ("/sensor/+", "/sensor/temperature3", true),
            ("/sensor/#", "/sensor/temperature3/tmpq", true),
            ("$share/group/topic/test", "/topic/test", true),
            (
                "$share/group/sensor/+/temperature",
                "/sensor/1/temperature",
                true,
            ),
            (
                "$share/group/sensor/+/temperature",
                "/sensor/1/2/temp",
                false,
            ),
            ("$share/group/sensor/+", "/sensor/temperature3", true),
            ("$share/group/sensor/#", "/sensor/temperature3/tmpq", true),
            ("y/+/z/#", "y/a/z/b", true),
        ];

        for (sub_pattern, topic, should_match) in test_cases {
            let result = is_match_sub_and_topic(sub_pattern, topic);
            assert_eq!(
                result.is_ok(),
                should_match,
                "Pattern '{}' vs topic '{}' failed",
                sub_pattern,
                topic
            );
        }
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
        assert_eq!(group_name, "comsumer1".to_string());
        assert_eq!(topic_name, "/finance/#".to_string());
    }

    #[tokio::test]
    async fn decode_sub_path_sub_test() {
        let path = "$share/group1/topic1/1".to_string();
        assert_eq!(decode_sub_path(&path), "/topic1/1".to_string());

        let path = "/topic1/1".to_string();
        assert_eq!(decode_sub_path(&path), "/topic1/1".to_string());

        let path = "$exclusive/topic1/1".to_string();
        assert_eq!(decode_sub_path(&path), "/topic1/1".to_string());
    }

    #[test]
    fn is_ignore_push_error_test() {
        // Should ignore these errors
        assert!(is_ignore_push_error(
            &MqttBrokerError::SessionNullSkipPushMessage("client1".to_string())
        ));
        assert!(is_ignore_push_error(
            &MqttBrokerError::ConnectionNullSkipPushMessage("client1".to_string())
        ));
        assert!(is_ignore_push_error(
            &MqttBrokerError::NotObtainAvailableConnection("client1".to_string(), 1000)
        ));
        assert!(is_ignore_push_error(&MqttBrokerError::OperationTimeout(
            1000,
            "op".to_string()
        )));

        // Should not ignore other errors
        assert!(!is_ignore_push_error(&MqttBrokerError::InvalidSubPath(
            "path".to_string()
        )));
    }

    #[test]
    fn is_error_by_suback_test() {
        // Success cases - should return false (no error)
        let suback = SubAck {
            pkid: 1,
            return_codes: vec![
                SubscribeReasonCode::Success(QoS::AtMostOnce),
                SubscribeReasonCode::QoS0,
            ],
        };
        assert!(!is_error_by_suback(&suback));

        let suback = SubAck {
            pkid: 1,
            return_codes: vec![
                SubscribeReasonCode::Success(QoS::AtLeastOnce),
                SubscribeReasonCode::QoS1,
            ],
        };
        assert!(!is_error_by_suback(&suback));

        let suback = SubAck {
            pkid: 1,
            return_codes: vec![
                SubscribeReasonCode::Success(QoS::ExactlyOnce),
                SubscribeReasonCode::QoS2,
            ],
        };
        assert!(!is_error_by_suback(&suback));

        // Error case - should return true
        let suback = SubAck {
            pkid: 1,
            return_codes: vec![SubscribeReasonCode::Unspecified],
        };
        assert!(is_error_by_suback(&suback));
    }

    #[test]
    fn build_sub_path_regex_error_test() {
        // Error: no wildcard
        assert!(build_sub_path_regex("/topic/test").is_err());

        // Error: # not at the end
        assert!(build_sub_path_regex("/topic/#/test").is_err());
        assert!(build_sub_path_regex("/topic/+/#/test").is_err());

        // Success: valid patterns
        assert!(build_sub_path_regex("/topic/#").is_ok());
        assert!(build_sub_path_regex("/topic/+").is_ok());
        assert!(build_sub_path_regex("/topic/+/#").is_ok());
    }

    #[tokio::test]
    async fn get_sub_topic_list_test() {
        let cache = test_build_mqtt_cache_manager().await;
        cache.add_topic("/test/topic1", &MQTTTopic::new("/test/topic1".to_string()));
        cache.add_topic("/test/topic2", &MQTTTopic::new("/test/topic2".to_string()));
        cache.add_topic("/other/topic", &MQTTTopic::new("/other/topic".to_string()));

        // Exact match
        let result = get_sub_topic_name_list(&cache, "/test/topic1").await;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "/test/topic1");

        // Wildcard #
        let result = get_sub_topic_name_list(&cache, "/test/#").await;
        assert_eq!(result.len(), 2);

        // Wildcard +
        let result = get_sub_topic_name_list(&cache, "/test/+").await;
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn min_qos_all_combinations_test() {
        // QoS0 vs QoS0
        assert_eq!(min_qos(QoS::AtMostOnce, QoS::AtMostOnce), QoS::AtMostOnce);

        // QoS0 vs QoS1
        assert_eq!(min_qos(QoS::AtMostOnce, QoS::AtLeastOnce), QoS::AtMostOnce);
        assert_eq!(min_qos(QoS::AtLeastOnce, QoS::AtMostOnce), QoS::AtMostOnce);

        // QoS0 vs QoS2
        assert_eq!(min_qos(QoS::AtMostOnce, QoS::ExactlyOnce), QoS::AtMostOnce);
        assert_eq!(min_qos(QoS::ExactlyOnce, QoS::AtMostOnce), QoS::AtMostOnce);

        // QoS1 vs QoS1
        assert_eq!(
            min_qos(QoS::AtLeastOnce, QoS::AtLeastOnce),
            QoS::AtLeastOnce
        );

        // QoS1 vs QoS2
        assert_eq!(
            min_qos(QoS::AtLeastOnce, QoS::ExactlyOnce),
            QoS::AtLeastOnce
        );
        assert_eq!(
            min_qos(QoS::ExactlyOnce, QoS::AtLeastOnce),
            QoS::AtLeastOnce
        );

        // QoS2 vs QoS2
        assert_eq!(
            min_qos(QoS::ExactlyOnce, QoS::ExactlyOnce),
            QoS::ExactlyOnce
        );
    }
}
