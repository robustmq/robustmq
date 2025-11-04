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
use common_base::{error::common::CommonError, tools::now_second, utils::serialize};
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
    pub user_properties: Option<Vec<(String, String)>>,
    pub subscription_identifiers: Option<Vec<usize>>,
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
            format_indicator: None,
            expiry_interval: 0,
            response_topic: None,
            correlation_data: None,
            user_properties: None,
            subscription_identifiers: None,
            content_type: None,
            create_time: now_second(),
        };

        match serialize::serialize(&message) {
            Ok(data) => Some(Record::from_bytes(data)),
            Err(e) => {
                error!("Message encoding failed, error message: {}", e);
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
            expiry_interval,
            create_time: now_second(),
            ..Default::default()
        };

        if let Some(properties) = publish_properties {
            message.format_indicator = properties.payload_format_indicator;
            message.response_topic = properties.response_topic.clone();
            message.correlation_data = properties.correlation_data.clone();
            message.content_type = properties.content_type.clone();

            if !properties.user_properties.is_empty() {
                message.user_properties = Some(properties.user_properties.clone());
            }
            if !properties.subscription_identifiers.is_empty() {
                message.subscription_identifiers =
                    Some(properties.subscription_identifiers.clone());
            }
        }

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
        match serialize::serialize(&msg) {
            Ok(data) => Some(Record::from_bytes(data)),
            Err(e) => {
                error!("Message encoding failed, error message: {}", e);
                None
            }
        }
    }

    pub fn decode_record(record: Record) -> Result<MqttMessage, CommonError> {
        serialize::deserialize(&record.data)
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn encode_str(&self) -> Result<String, CommonError> {
        let bytes = serialize::serialize(self)?;
        String::from_utf8(bytes)
            .map_err(|e| CommonError::CommonError(format!("Failed to convert to UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use protocol::mqtt::common::{Publish, PublishProperties, QoS};

    #[test]
    fn test_simple_serialize() {
        // Try with AtLeastOnce instead
        let msg = MqttMessage {
            client_id: "-".to_string(),
            dup: true,
            qos: QoS::AtLeastOnce,
            pkid: 42,
            retain: false,
            topic: Bytes::from("topic"),
            payload: Bytes::from("payload"),
            format_indicator: Some(1),
            expiry_interval: 3600,
            response_topic: Some("response".to_string()),
            correlation_data: Some(Bytes::from("corr")),
            user_properties: Some(vec![("k".to_string(), "v".to_string())]),
            subscription_identifiers: Some(vec![1]),
            content_type: Some("text".to_string()),
            create_time: 12345,
        };

        // Direct serialize/deserialize
        let data = serialize::serialize(&msg).unwrap();
        println!("Serialized {} bytes", data.len());

        let decoded: MqttMessage = serialize::deserialize(&data).unwrap();
        assert_eq!(decoded.client_id, msg.client_id);
        assert_eq!(decoded.topic, msg.topic);
    }

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
        assert!(msg.user_properties.is_none());
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
        assert!(msg.user_properties.is_some());
        let user_props = msg.user_properties.as_ref().unwrap();
        assert_eq!(user_props.len(), 1);
        assert_eq!(user_props[0].0, "key1");
        assert_eq!(user_props[0].1, "value1");
        assert_eq!(msg.subscription_identifiers, Some(vec![1, 2, 3]));
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
        assert!(msg.user_properties.is_none());
        assert!(msg.subscription_identifiers.is_none());
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
            user_properties: Some(vec![("key1".to_string(), "value1".to_string())]),
            subscription_identifiers: Some(vec![1, 2, 3]),
            content_type: Some("application/json".to_string()),
            create_time: now_second(),
        };

        let encoded = msg.encode().unwrap();
        let record = Record::from_bytes(encoded);
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
