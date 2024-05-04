use crate::{
    metadata::cache::MetadataCacheManager, server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::{errors::RobustMQError, log::error};
use protocol::mqtt::{
    MQTTPacket, Publish, PublishProperties, QoS, RetainForwardRule, Subscribe, SubscribeProperties,
};
use regex::Regex;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::Sender;

pub fn path_regex_match(topic_name: String, sub_regex: String) -> bool {
    // Path perfect matching
    if topic_name == sub_regex {
        return true;
    }

    if sub_regex.contains("+") {
        let sub_regex = sub_regex.replace("+", "[^+*/]+");
        let re = Regex::new(&format!("{}", sub_regex)).unwrap();
        println!("{}", sub_regex);
        return re.is_match(&topic_name);
    }

    if sub_regex.contains("#") {
        if sub_regex.split("#").last().unwrap() != "#".to_string() {
            return false;
        }
        let sub_regex = sub_regex.replace("#", "[^+#]+");
        let re = Regex::new(&format!("{}", sub_regex)).unwrap();
        return re.is_match(&topic_name);
    }

    return false;
}

// Reservation messages are processed when a subscription is created
pub async fn send_retain_message<T, S>(
    connect_id: u64,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    message_storage: MessageStorage<S>,
    metadata_cache: Arc<MetadataCacheManager<T>>,
    response_queue_sx: Sender<ResponsePackage>,
    new_sub: bool,
    dup_msg: bool,
) -> Result<(), RobustMQError>
where
    T: StorageAdapter + Send + Sync + 'static,
    S: StorageAdapter + Send + Sync + 'static,
{
    let mut sub_id = Vec::new();
    if let Some(properties) = subscribe_properties {
        if let Some(id) = properties.subscription_identifier {
            sub_id.push(id);
        }
    }

    for filter in subscribe.filters {
        if filter.retain_forward_rule == RetainForwardRule::Never {
            continue;
        }

        if filter.retain_forward_rule == RetainForwardRule::OnNewSubscribe && !new_sub {
            continue;
        }

        let topic_id_list = get_sub_topic_id_list(metadata_cache.clone(), filter.path).await;
        for topic_id in topic_id_list {
            match message_storage.get_retain_message(topic_id.clone()).await {
                Ok(Some(msg)) => {
                    if let Some(topic_name) = metadata_cache.topic_name_by_id(topic_id) {
                        let publish = Publish {
                            dup: dup_msg,
                            qos: max_qos(msg.qos, filter.qos),
                            pkid: subscribe.packet_identifier,
                            retain: false,
                            topic: Bytes::from(topic_name),
                            payload: msg.payload,
                        };
                        let properties = PublishProperties {
                            payload_format_indicator: None,
                            message_expiry_interval: None,
                            topic_alias: None,
                            response_topic: None,
                            correlation_data: None,
                            user_properties: Vec::new(),
                            subscription_identifiers: sub_id.clone(),
                            content_type: None,
                        };

                        let resp = ResponsePackage {
                            connection_id: connect_id,
                            packet: MQTTPacket::Publish(publish, Some(properties)),
                        };
                        match response_queue_sx.send(resp) {
                            Ok(_) => {}
                            Err(e) => error(format!("{}", e.to_string())),
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }
    }
    return Ok(());
}

pub fn max_qos(msg_qos: QoS, sub_max_qos: QoS) -> QoS {
    if msg_qos <= sub_max_qos {
        return msg_qos;
    }
    return sub_max_qos;
}

pub async fn get_sub_topic_id_list<T>(
    metadata_cache: Arc<MetadataCacheManager<T>>,
    sub_path: String,
) -> Vec<String> {
    let topic_id_name = metadata_cache.topic_id_name.clone();

    let mut result = Vec::new();
    for (topic_id, topic_name) in topic_id_name {
        if path_regex_match(topic_name.clone(), sub_path.clone()) {
            result.push(topic_id);
        }
    }
    return result;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use protocol::mqtt::{Filter, MQTTPacket, QoS, Subscribe, SubscribeProperties};
    use storage_adapter::memory::MemoryStorageAdapter;
    use tokio::sync::broadcast;

    use crate::{
        handler::subscribe::{
            get_sub_topic_id_list, max_qos, path_regex_match, send_retain_message,
        },
        metadata::{cache::MetadataCacheManager, message::Message, topic::Topic},
        storage::message::MessageStorage,
    };

    #[test]
    fn path_regex_match_test() {
        let topic_name = "/topic/test".to_string();
        let sub_regex = "/topic/test".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"sensor/1/temperature".to_string();
        let sub_regex = r"sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);

        let topic_name = r"sensor/1/2/temperature3".to_string();
        let sub_regex = r"sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), false);

        let topic_name = r"sensor/temperature3".to_string();
        let sub_regex = r"sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), false);

        let topic_name = r"sensor/temperature3".to_string();
        let sub_regex = r"sensor/+".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);

        let topic_name = r"sensor/temperature3/tmpq".to_string();
        let sub_regex = r"sensor/+".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), false);
    }

    #[test]
    fn max_qos_test() {
        let mut sub_max_qos = QoS::AtMostOnce;
        let mut msg_qos = QoS::AtLeastOnce;
        assert_eq!(max_qos(msg_qos, sub_max_qos), sub_max_qos);

        msg_qos = QoS::AtMostOnce;
        sub_max_qos = QoS::AtLeastOnce;
        assert_eq!(max_qos(msg_qos, sub_max_qos), msg_qos);
    }

    #[tokio::test]
    async fn get_sub_topic_list_test() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(MetadataCacheManager::new(
            storage_adapter.clone(),
            "test-cluster".to_string(),
        ));
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        metadata_cache.set_topic(&topic_name, &topic);

        let sub_path = "/test/topic".to_string();
        let result = get_sub_topic_id_list(metadata_cache.clone(), sub_path).await;
        assert!(result.len() == 1);
        assert_eq!(result.get(0).unwrap().clone(), topic.topic_id);
    }

    #[tokio::test]
    async fn send_retain_message_test() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(MetadataCacheManager::new(
            storage_adapter.clone(),
            "test-cluster".to_string(),
        ));
        let (response_queue_sx, mut response_queue_rx) = broadcast::channel(1000);
        let connect_id = 1;
        let mut filters = Vec::new();
        let flt = Filter {
            path: "/test/topic".to_string(),
            qos: QoS::AtLeastOnce,
            nolocal: true,
            preserve_retain: true,
            retain_forward_rule: protocol::mqtt::RetainForwardRule::OnEverySubscribe,
        };
        filters.push(flt);
        let subscribe = Subscribe {
            packet_identifier: 1,
            filters,
        };
        let subscribe_properties = Some(SubscribeProperties::default());
        let message_storage = MessageStorage::new(storage_adapter.clone());
        let new_sub = true;
        let dup_msg = true;

        let topic_name = "/test/topic".to_string();
        let payload = "testtesttest".to_string();
        let topic = Topic::new(&topic_name);

        metadata_cache.set_topic(&topic_name, &topic);

        let mut retain_message = Message::default();
        retain_message.dup = false;
        retain_message.qos = QoS::AtLeastOnce;
        retain_message.pkid = 1;
        retain_message.retain = true;
        retain_message.topic = Bytes::from(topic_name.clone());
        retain_message.payload = Bytes::from(payload);

        match message_storage
            .save_retain_message(topic.topic_id, retain_message.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }

        match send_retain_message(
            connect_id,
            subscribe,
            subscribe_properties,
            message_storage,
            metadata_cache.clone(),
            response_queue_sx.clone(),
            new_sub,
            dup_msg,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }

        loop {
            match response_queue_rx.recv().await {
                Ok(packet) => {
                    if let MQTTPacket::Publish(publish, _) = packet.packet {
                        assert_eq!(publish.topic, retain_message.topic);
                        assert_eq!(publish.payload, retain_message.payload);
                    } else {
                        println!("Package does not exist");
                        assert!(false);
                    }
                    break;
                }
                Err(e) => {
                    println!("{}", e)
                }
            }
        }
    }
}
