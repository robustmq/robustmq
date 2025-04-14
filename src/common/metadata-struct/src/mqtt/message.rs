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
            pkid: publish.pkid,
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
