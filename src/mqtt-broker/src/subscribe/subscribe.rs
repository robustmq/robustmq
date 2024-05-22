use crate::core::metadata_cache::MetadataCacheManager;
use crate::qos::ack_manager::{AckManager, AckPacketInfo};
use crate::server::MQTTProtocol;
use crate::{server::tcp::packet::ResponsePackage, storage::message::MessageStorage};
use bytes::Bytes;
use clients::placement::mqtt::call::placement_get_share_sub;
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::tools::now_second;
use common_base::{errors::RobustMQError, log::error};
use protocol::mqtt::{
    MQTTPacket, Publish, PublishProperties, QoS, RetainForwardRule, Subscribe, SubscribeProperties,
};
use protocol::placement_center::generate::mqtt::{GetShareSubReply, GetShareSubRequest};
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::mpsc;
use tokio::time::timeout;

const SHARE_SUB_PREFIX: &str = "$share";
const SHARE_SUB_REWRITE_PUBLISH_FLAG: &str = "$system_ssrpf";
const SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE: &str = "True";

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
pub async fn save_retain_message<S>(
    connect_id: u64,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    message_storage: MessageStorage<S>,
    metadata_cache: Arc<MetadataCacheManager>,
    response_queue_sx: Sender<ResponsePackage>,
    new_sub: bool,
    dup_msg: bool,
) -> Result<(), RobustMQError>
where
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
                            qos: min_qos(msg.qos, filter.qos),
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

pub fn min_qos(qos: QoS, sub_qos: QoS) -> QoS {
    if qos <= sub_qos {
        return qos;
    }
    return sub_qos;
}

pub async fn get_sub_topic_id_list(
    metadata_cache: Arc<MetadataCacheManager>,
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

pub fn is_share_sub(sub_name: String) -> bool {
    return sub_name.starts_with(SHARE_SUB_PREFIX);
}

pub fn decode_share_info(sub_name: String) -> (String, String) {
    let mut str_slice: Vec<&str> = sub_name.split("/").collect();
    str_slice.remove(0);
    let group_name = str_slice.remove(0).to_string();
    let sub_name = format!("/{}", str_slice.join("/"));
    return (group_name, sub_name);
}

pub fn is_contain_rewrite_flag(user_properties: Vec<(String, String)>) -> bool {
    // source IP

    //
    for (k, v) in user_properties {
        if k == SHARE_SUB_REWRITE_PUBLISH_FLAG
            && v == SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE.to_string()
        {
            return true;
        }
    }

    return false;
}

pub async fn get_share_sub_leader(
    client_poll: Arc<ClientPool>,
    group_name: String,
    sub_name: String,
) -> Result<GetShareSubReply, RobustMQError> {
    let conf = broker_mqtt_conf();
    let req = GetShareSubRequest {
        cluster_name: conf.cluster_name.clone(),
        group_name,
        sub_name: sub_name.clone(),
    };
    match placement_get_share_sub(client_poll, conf.placement.server.clone(), req).await {
        Ok(reply) => {
            return Ok(reply);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub fn share_sub_rewrite_publish_flag() -> (String, String) {
    return (
        SHARE_SUB_REWRITE_PUBLISH_FLAG.to_string(),
        SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE.to_string(),
    );
}

pub async fn retry_publish(
    client_id: String,
    pkid: u16,
    metadata_cache: Arc<MetadataCacheManager>,
    ack_manager: Arc<AckManager>,
    qos: QoS,
    protocol: MQTTProtocol,
    resp: ResponsePackage,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
) -> Result<(), RobustMQError> {
    loop {
        match publish_to_client(
            protocol.clone(),
            resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                match qos {
                    protocol::mqtt::QoS::AtMostOnce => {
                        return Ok(());
                    }
                    
                    // protocol::mqtt::QoS::AtLeastOnce
                    protocol::mqtt::QoS::AtLeastOnce => {

                    }
                    
                    // protocol::mqtt::QoS::ExactlyOnce
                    protocol::mqtt::QoS::ExactlyOnce => {
                        metadata_cache.save_pkid_info(client_id.clone(), pkid);

                        let (qos_sx, qos_rx) = mpsc::channel(1);
                        ack_manager.add(
                            client_id.clone(),
                            pkid,
                            AckPacketInfo {
                                sx: qos_sx,
                                create_time: now_second(),
                            },
                        );

                        if wait_packet_ack(qos_rx).await {
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => {
                error(e.to_string());
            }
        };
    }
}

pub async fn wait_packet_ack(mut rx: mpsc::Receiver<bool>) -> bool {
    let res = timeout(Duration::from_secs(30), async {
        if let Some(flag) = rx.recv().await {
            return flag;
        }
        return false;
    });
    match res.await {
        Ok(_) => return true,
        Err(_) => {
            return false;
        }
    }
}

pub async fn publish_to_client(
    protocol: MQTTProtocol,
    resp: ResponsePackage,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) -> Result<(), RobustMQError> {
    if protocol == MQTTProtocol::MQTT4 {
        match response_queue_sx4.send(resp) {
            Ok(_) => {}
            Err(e) => return Err(RobustMQError::CommmonError(format!("{}", e.to_string()))),
        }
    } else if protocol == MQTTProtocol::MQTT5 {
        match response_queue_sx5.send(resp) {
            Ok(_) => {}
            Err(e) => return Err(RobustMQError::CommmonError(format!("{}", e.to_string()))),
        }
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use protocol::mqtt::{Filter, MQTTPacket, QoS, Subscribe, SubscribeProperties};
    use storage_adapter::memory::MemoryStorageAdapter;
    use tokio::sync::broadcast;

    use crate::core::metadata_cache::MetadataCacheManager;
    use crate::subscribe::subscribe::{decode_share_info, is_share_sub};
    use crate::{
        metadata::{message::Message, topic::Topic},
        storage::message::MessageStorage,
        subscribe::subscribe::{
            get_sub_topic_id_list, min_qos, path_regex_match, save_retain_message,
        },
    };

    #[tokio::test]
    async fn is_share_sub_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        assert!(is_share_sub(sub1));
        assert!(is_share_sub(sub2));
        assert!(is_share_sub(sub3));
        assert!(is_share_sub(sub4));

        let sub5 = "/comsumer1/$share/finance/#".to_string();
        let sub6 = "/comsumer1/$share/finance/$share".to_string();

        assert!(!is_share_sub(sub5));
        assert!(!is_share_sub(sub6));
    }

    #[tokio::test]
    async fn decode_share_info_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        let (group_name, topic_name) = decode_share_info(sub1);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(sub2);
        assert_eq!(group_name, "consumer2".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(sub3);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/#".to_string());

        let (group_name, topic_name) = decode_share_info(sub4);
        assert_eq!(group_name, "comsumer1".to_string());
        assert_eq!(topic_name, "/finance/#".to_string());
    }
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
        assert_eq!(min_qos(msg_qos, sub_max_qos), sub_max_qos);

        msg_qos = QoS::AtMostOnce;
        sub_max_qos = QoS::AtLeastOnce;
        assert_eq!(min_qos(msg_qos, sub_max_qos), msg_qos);
    }

    #[tokio::test]
    async fn get_sub_topic_list_test() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(MetadataCacheManager::new("test-cluster".to_string()));
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        metadata_cache.add_topic(&topic_name, &topic);

        let sub_path = "/test/topic".to_string();
        let result = get_sub_topic_id_list(metadata_cache.clone(), sub_path).await;
        assert!(result.len() == 1);
        assert_eq!(result.get(0).unwrap().clone(), topic.topic_id);
    }

    #[tokio::test]
    async fn send_retain_message_test() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(MetadataCacheManager::new("test-cluster".to_string()));
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

        metadata_cache.add_topic(&topic_name, &topic);

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

        match save_retain_message(
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
