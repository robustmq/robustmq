use bytes::Bytes;
use common_base::tools::now_mills;
use protocol::mqtt::{Publish, PublishProperties, QoS};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Message {
    dup: bool,
    qos: QoS,
    pkid: u16,
    retain: bool,
    topic: Bytes,
    payload: Bytes,
    format_indicator: Option<u8>,
    expiry_interval: Option<u32>,
    response_topic: Option<String>,
    correlation_data: Option<Bytes>,
    user_properties: Vec<(String, String)>,
    subscription_identifiers: Vec<usize>,
    content_type: Option<String>,
    create_time: u128,
}

impl Message {
    pub fn build_message(
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> Message {
        let mut message = Message::default();
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
        publish: Publish,
        publish_properties: Option<PublishProperties>){
        let msg = Message::build_message(publish, publish_properties);
            
    }
}
