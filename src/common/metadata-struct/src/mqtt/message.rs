use bytes::Bytes;
use common_base::{errors::RobustMQError, tools::now_mills};
use protocol::mqtt::{Publish, PublishProperties, QoS};
use serde::{Deserialize, Serialize};
use storage_adapter::record::Record;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MQTTMessage {
    pub client_id: String,
    pub dup: bool,
    pub qos: QoS,
    pub pkid: u16,
    pub retain: bool,
    pub topic: Bytes,
    pub payload: Bytes,
    pub format_indicator: Option<u8>,
    pub expiry_interval: Option<u32>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
    pub subscription_identifiers: Vec<usize>,
    pub content_type: Option<String>,
    pub create_time: u128,
}

impl MQTTMessage {
    pub fn build_message(
        client_id: String,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> MQTTMessage {
        let mut message = MQTTMessage::default();
        message.client_id = client_id;
        message.dup = publish.dup;
        message.qos = publish.qos;
        message.pkid = publish.pkid;
        message.retain = publish.retain;
        message.topic = publish.topic;
        message.payload = publish.payload;
        if let Some(properties) = publish_properties {
            message.format_indicator = properties.payload_format_indicator;
            message.expiry_interval = properties.message_expiry_interval;
            message.response_topic = properties.response_topic;
            message.correlation_data = properties.correlation_data;
            message.user_properties = properties.user_properties;
            message.subscription_identifiers = properties.subscription_identifiers;
            message.content_type = properties.content_type;
        }
        message.create_time = now_mills();
        return message;
    }

    pub fn build_record(
        client_id: String,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> Option<Record> {
        let msg = MQTTMessage::build_message(client_id, publish, publish_properties);
        match serde_json::to_vec(&msg) {
            Ok(data) => {
                return Some(Record::build_b(data));
            }

            Err(_) => {
                return None;
            }
        }
    }

    pub fn decode_record(record: Record) -> Result<MQTTMessage, RobustMQError> {
        let data: MQTTMessage = match serde_json::from_slice(record.data.as_slice()) {
            Ok(da) => da,
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        };
        return Ok(data);
    }

    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }

}
