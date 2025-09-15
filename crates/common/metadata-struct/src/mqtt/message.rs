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

use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use protocol::mqtt::common::{Publish, PublishProperties, QoS};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::adapter::record::Record;

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct MqttMessage {
    pub client_id: String,
    pub dup: bool,
    pub qos: QoS,
    pub pkid: u16,
    pub retain: bool,
    pub topic: Bytes,
    pub payload: Bytes,
    pub format_indicator: Option<u8>,
    pub expiry_interval: u64,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
    pub subscription_identifiers: Vec<usize>,
    pub content_type: Option<String>,
    pub create_time: u64,
}

impl MqttMessage {
    pub fn build_system_topic_message(topic_name: String, payload: String) -> Option<Record> {
        let message = MqttMessage {
            client_id: "-".to_string(),
            dup: false,
            qos: QoS::AtMostOnce,
            pkid: 0,
            retain: false,
            topic: Bytes::from(topic_name),
            payload: Bytes::from(payload),
            create_time: now_second(),
            ..Default::default()
        };

        match serde_json::to_vec(&message) {
            Ok(data) => Some(Record::build_byte(data)),

            Err(e) => {
                error!("Message encoding failed, error message :{}", e.to_string());
                None
            }
        }
    }

    pub fn build_message(
        client_id: &str,
        publish: &Publish,
        publish_properties: &Option<PublishProperties>,
        expiry_interval: u64,
    ) -> MqttMessage {
        let mut message = MqttMessage {
            client_id: client_id.to_owned(),
            dup: publish.dup,
            qos: publish.qos,
            pkid: publish.p_kid,
            retain: publish.retain,
            topic: publish.topic.clone(),
            payload: publish.payload.clone(),
            ..Default::default()
        };
        if let Some(properties) = publish_properties {
            message.format_indicator = properties.payload_format_indicator;
            message.expiry_interval = expiry_interval;
            message.response_topic = properties.response_topic.clone();
            message.correlation_data = properties.correlation_data.clone();
            message.user_properties = properties.user_properties.clone();
            message.subscription_identifiers = properties.subscription_identifiers.clone();
            message.content_type = properties.content_type.clone();
        } else {
            message.format_indicator = None;
            message.expiry_interval = expiry_interval;
            message.response_topic = None;
            message.correlation_data = None;
            message.user_properties = Vec::new();
            message.subscription_identifiers = Vec::new();
            message.content_type = None;
        }
        message.create_time = now_second();
        message
    }

    pub fn build_record(
        client_id: &str,
        publish: &Publish,
        publish_properties: &Option<PublishProperties>,
        expiry_interval: u64,
    ) -> Option<Record> {
        let msg =
            MqttMessage::build_message(client_id, publish, publish_properties, expiry_interval);
        match serde_json::to_vec(&msg) {
            Ok(data) => Some(Record::build_byte(data)),

            Err(e) => {
                error!("Message encoding failed, error message :{}", e.to_string());
                None
            }
        }
    }

    pub fn decode_record(record: Record) -> Result<MqttMessage, CommonError> {
        let data: MqttMessage = match serde_json::from_slice(record.data.as_slice()) {
            Ok(da) => da,
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        };
        Ok(data)
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use protocol::mqtt::common::{Publish, PublishProperties, QoS};

    #[test]
    fn test_build_system_topic_message() {
        let topic = "sys/topic".to_string();
        let payload = "payload-data".to_string();
        let record = MqttMessage::build_system_topic_message(topic.clone(), payload.clone());

        assert!(record.is_some());
        let rec = record.unwrap();
        let msg = MqttMessage::decode_record(rec).unwrap();

        assert_eq!(msg.client_id, "-");
        assert!(!msg.retain);
        assert_eq!(msg.qos, QoS::AtMostOnce);
        assert_eq!(msg.pkid, 0);
        assert!(!msg.retain);
        assert_eq!(msg.topic, Bytes::from(topic));
        assert_eq!(msg.payload, Bytes::from(payload));
        assert_eq!(msg.format_indicator, None);
        assert_eq!(msg.user_properties.len(), 0);
        assert!(msg.create_time > 0);
    }

    #[test]
    fn test_build_message() {
        let client_id = "test-client";
        let publish = Publish {
            dup: true,
            qos: QoS::AtLeastOnce,
            p_kid: 42,
            retain: true,
            topic: Bytes::from("test/topic"),
            payload: Bytes::from("hello world"),
        };

        let props = Some(PublishProperties {
            payload_format_indicator: Some(1),
            message_expiry_interval: None,
            topic_alias: None,
            response_topic: Some("response/topic".to_string()),
            correlation_data: Some(Bytes::from("correlation-data")),
            user_properties: vec![("key1".to_string(), "value1".to_string())],
            subscription_identifiers: vec![1, 2, 3],
            content_type: Some("application/json".to_string()),
        });

        let expiry_interval = 3600;
        let msg = MqttMessage::build_message(client_id, &publish, &props, expiry_interval);

        assert_eq!(msg.client_id, client_id);
        assert_eq!(msg.dup, publish.dup);
        assert_eq!(msg.qos, publish.qos);
        assert_eq!(msg.pkid, publish.p_kid);
        assert_eq!(msg.retain, publish.retain);
        assert_eq!(msg.topic, publish.topic);
        assert_eq!(msg.payload, publish.payload);
        assert_eq!(msg.format_indicator, Some(1));
        assert_eq!(msg.expiry_interval, expiry_interval);
        assert_eq!(msg.response_topic, Some("response/topic".to_string()));
        assert_eq!(msg.correlation_data, Some(Bytes::from("correlation-data")));
        assert_eq!(msg.user_properties.len(), 1);
        assert_eq!(msg.user_properties[0].0, "key1");
        assert_eq!(msg.user_properties[0].1, "value1");
        assert_eq!(msg.subscription_identifiers, vec![1, 2, 3]);
        assert_eq!(msg.content_type, Some("application/json".to_string()));
        assert!(msg.create_time > 0);
    }

    #[test]
    fn test_build_message_without_properties() {
        let client_id = "test-client";
        let publish = Publish {
            dup: false,
            qos: QoS::ExactlyOnce,
            p_kid: 123,
            retain: false,
            topic: Bytes::from("test/topic"),
            payload: Bytes::from("hello world"),
        };

        let expiry_interval = 0;
        let msg = MqttMessage::build_message(client_id, &publish, &None, expiry_interval);

        assert_eq!(msg.client_id, client_id);
        assert_eq!(msg.dup, publish.dup);
        assert_eq!(msg.qos, publish.qos);
        assert_eq!(msg.pkid, publish.p_kid);
        assert_eq!(msg.retain, publish.retain);
        assert_eq!(msg.topic, publish.topic);
        assert_eq!(msg.payload, publish.payload);
        assert_eq!(msg.format_indicator, None);
        assert_eq!(msg.expiry_interval, expiry_interval);
        assert_eq!(msg.response_topic, None);
        assert_eq!(msg.correlation_data, None);
        assert_eq!(msg.user_properties.len(), 0);
        assert_eq!(msg.subscription_identifiers.len(), 0);
        assert_eq!(msg.content_type, None);
        assert!(msg.create_time > 0);
    }

    #[test]
    fn test_build_record() {
        let client_id = "test-client";
        let publish = Publish {
            dup: false,
            qos: QoS::AtMostOnce,
            p_kid: 0,
            retain: false,
            topic: Bytes::from("test/topic"),
            payload: Bytes::from("test message"),
        };

        let record = MqttMessage::build_record(client_id, &publish, &None, 0);
        assert!(record.is_some());

        let decoded = MqttMessage::decode_record(record.unwrap()).unwrap();
        assert_eq!(decoded.client_id, client_id);
        assert_eq!(decoded.topic, publish.topic);
        assert_eq!(decoded.payload, publish.payload);
    }

    #[test]
    fn test_encode_decode() {
        let msg = MqttMessage {
            client_id: "test-client".to_string(),
            dup: true,
            qos: QoS::AtLeastOnce,
            pkid: 42,
            retain: true,
            topic: Bytes::from("test/topic"),
            payload: Bytes::from("test message"),
            format_indicator: Some(1),
            expiry_interval: 3600,
            response_topic: Some("response/topic".to_string()),
            correlation_data: Some(Bytes::from("correlation-data")),
            user_properties: vec![("key1".to_string(), "value1".to_string())],
            subscription_identifiers: vec![1, 2, 3],
            content_type: Some("application/json".to_string()),
            create_time: now_second(),
        };

        let encoded = msg.encode();
        let record = Record::build_byte(encoded);
        let decoded = MqttMessage::decode_record(record).unwrap();

        assert_eq!(decoded.client_id, msg.client_id);
        assert_eq!(decoded.dup, msg.dup);
        assert_eq!(decoded.qos, msg.qos);
        assert_eq!(decoded.pkid, msg.pkid);
        assert_eq!(decoded.retain, msg.retain);
        assert_eq!(decoded.topic, msg.topic);
        assert_eq!(decoded.payload, msg.payload);
        assert_eq!(decoded.format_indicator, msg.format_indicator);
        assert_eq!(decoded.expiry_interval, msg.expiry_interval);
        assert_eq!(decoded.response_topic, msg.response_topic);
        assert_eq!(decoded.correlation_data, msg.correlation_data);
        assert_eq!(decoded.user_properties, msg.user_properties);
        assert_eq!(
            decoded.subscription_identifiers,
            msg.subscription_identifiers
        );
        assert_eq!(decoded.content_type, msg.content_type);
        assert_eq!(decoded.create_time, msg.create_time);
    }
}
