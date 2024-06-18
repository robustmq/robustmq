use crate::core::metadata_cache::MetadataCacheManager;
use crate::core::qos_manager::QosAckPackageData;
use crate::server::MQTTProtocol;
use crate::storage::topic::TopicStorage;
use crate::{server::tcp::packet::ResponsePackage, storage::message::MessageStorage};
use bytes::Bytes;
use clients::placement::mqtt::call::placement_get_share_sub_leader;
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::{errors::RobustMQError, log::error};
use protocol::mqtt::{
    MQTTPacket, PubRel, Publish, PublishProperties, QoS, RetainForwardRule, Subscribe,
    SubscribeProperties,
};
use protocol::placement_center::generate::mqtt::{
    GetShareSubLeaderReply, GetShareSubLeaderRequest,
};
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::{sleep, timeout};

const SHARE_SUB_PREFIX: &str = "$share";

pub fn sub_path_validator(sub_path: String) -> bool {
    let regex = Regex::new(r"^[\$a-zA-Z0-9_#+/]+$").unwrap();

    if !regex.is_match(&sub_path) {
        return false;
    }

    for path in sub_path.split("/") {
        if path.contains("+") && path != "+" {
            return false;
        }
        if path.contains("#") && path != "#" {
            return false;
        }
    }

    return true;
}

pub fn path_regex_match(topic_name: String, sub_path: String) -> bool {
    let path = if is_share_sub(sub_path.clone()) {
        let (group_name, group_path) = decode_share_info(sub_path);
        group_path
    } else {
        sub_path
    };

    // Path perfect matching
    if topic_name == path {
        return true;
    }

    if path.contains("+") {
        let sub_regex = path.replace("+", "[^+*/]+");
        let re = Regex::new(&format!("{}", sub_regex)).unwrap();
        return re.is_match(&topic_name);
    }

    if path.contains("#") {
        if path.split("/").last().unwrap() != "#".to_string() {
            return false;
        }
        let sub_regex = path.replace("#", "[^+#]+");
        let re = Regex::new(&format!("{}", sub_regex)).unwrap();
        return re.is_match(&topic_name);
    }

    return false;
}

// Reservation messages are processed when a subscription is created
pub async fn send_retain_message(
    connect_id: u64,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    client_poll: Arc<ClientPool>,
    metadata_cache: Arc<MetadataCacheManager>,
    response_queue_sx: Sender<ResponsePackage>,
    new_sub: bool,
    dup_msg: bool,
) -> Result<(), RobustMQError> {
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
        let topic_storage = TopicStorage::new(client_poll.clone());
        for topic_id in topic_id_list {
            match topic_storage.get_retain_message(topic_id.clone()).await {
                Ok(msg) => {
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
                        // todo send retain
                        match response_queue_sx.send(resp) {
                            Ok(_) => {}
                            Err(e) => error(format!("{}", e.to_string())),
                        }
                    }
                }
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
    let mut result = Vec::new();
    for (topic_id, topic_name) in metadata_cache.topic_id_name.clone() {
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

pub async fn get_share_sub_leader(
    client_poll: Arc<ClientPool>,
    group_name: String,
) -> Result<GetShareSubLeaderReply, RobustMQError> {
    let conf = broker_mqtt_conf();
    let req = GetShareSubLeaderRequest {
        cluster_name: conf.cluster_name.clone(),
        group_name,
    };
    match placement_get_share_sub_leader(client_poll, conf.placement.server.clone(), req).await {
        Ok(reply) => {
            return Ok(reply);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn wait_packet_ack(sx: Sender<QosAckPackageData>) -> Option<QosAckPackageData> {
    let res = timeout(Duration::from_secs(120), async {
        match sx.subscribe().recv().await {
            Ok(data) => {
                return Some(data);
            }
            Err(_) => {
                return None;
            }
        }
    });

    match res.await {
        Ok(data) => data,
        Err(_) => {
            return None;
        }
    }
}

pub async fn publish_to_response_queue(
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

pub async fn qos2_send_publish(
    metadata_cache: Arc<MetadataCacheManager>,
    client_id: String,
    mut publish: Publish,
    publish_properties: Option<PublishProperties>,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut retry_times = 0;
    let mut stop_rx = stop_sx.subscribe();
    loop {
        let connect_id = if let Some(id) = metadata_cache.get_connect_id(client_id.clone()) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        retry_times = retry_times + 1;
        publish.dup = retry_times >= 2;

        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::Publish(publish.clone(), publish_properties.clone()),
        };

        select! {
            val = stop_rx.recv() => {
                match val{
                    Ok(flag) => {
                        if flag {
                            return;
                        }
                    }
                    Err(_) => {}
                }
            }
            val = publish_to_response_queue(
                protocol.clone(),
                resp.clone(),
                response_queue_sx4.clone(),
                response_queue_sx5.clone(),
            ) =>{
                match val{
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        error(format!(
                            "Failed to write QOS2 Publish message to response queue, failure message: {}",
                            e.to_string()
                        ));
                        sleep(Duration::from_millis(1)).await;
                    }
                }
            }
        }
    }
}

pub async fn qos2_send_pubrel(
    metadata_cache: Arc<MetadataCacheManager>,
    client_id: String,
    pkid: u16,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();

    loop {
        let connect_id = if let Some(id) = metadata_cache.get_connect_id(client_id.clone()) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        let pubrel = PubRel {
            pkid,
            reason: protocol::mqtt::PubRelReason::Success,
        };

        let pubrel_resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::PubRel(pubrel, None),
        };

        select! {
            val = stop_rx.recv() => {
                match val{
                    Ok(flag) => {
                        if flag {
                            return;
                        }
                    }
                    Err(_) => {}
                }
            }

            val = publish_to_response_queue(
                protocol.clone(),
                pubrel_resp.clone(),
                response_queue_sx4.clone(),
                response_queue_sx5.clone(),
            ) =>{
                match val{
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        error(format!(
                            "Failed to write PubRel message to response queue, failure message: {}",
                            e.to_string()
                        ));
                    }
                }
            }

        }
    }
}

pub async fn loop_commit_offset<S>(
    message_storage: MessageStorage<S>,
    topic_id: String,
    group_id: String,
    offset: u128,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    loop {
        match message_storage
            .commit_group_offset(topic_id.clone(), group_id.clone(), offset)
            .await
        {
            Ok(_) => {
                break;
            }
            Err(e) => {
                error(e.to_string());
            }
        }
    }
}

// When the subscription QOS is 0,
// the message can be pushed directly to the request return queue without the need for a retry mechanism.
pub async fn publish_message_qos0(
    metadata_cache: Arc<MetadataCacheManager>,
    mqtt_client_id: String,
    publish: Publish,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
) {
    let connect_id;
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return;
                }
            }
            Err(_) => {}
        }

        if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id.clone()) {
            connect_id = id;
            break;
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };
    }

    let resp = ResponsePackage {
        connection_id: connect_id,
        packet: MQTTPacket::Publish(publish, None),
    };

    // 2. publish to mqtt client
    match publish_to_response_queue(
        protocol.clone(),
        resp.clone(),
        response_queue_sx4.clone(),
        response_queue_sx5.clone(),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            error(format!(
                "Failed Publish message to response queue, failure message: {}",
                e.to_string()
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use clients::poll::ClientPool;
    use metadata_struct::mqtt::message::MQTTMessage;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::{Filter, MQTTPacket, QoS, Subscribe, SubscribeProperties};
    use storage_adapter::memory::MemoryStorageAdapter;
    use tokio::sync::broadcast;

    use crate::core::metadata_cache::MetadataCacheManager;
    use crate::storage::topic::TopicStorage;
    use crate::subscribe::sub_common::{decode_share_info, is_share_sub, sub_path_validator};
    use crate::{
        storage::message::MessageStorage,
        subscribe::sub_common::{
            get_sub_topic_id_list, min_qos, path_regex_match, send_retain_message,
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
        let topic_name = "/loboxu/test".to_string();
        let sub_regex = "/loboxu/#".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = "/topic/test".to_string();
        let sub_regex = "/topic/test".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/1/temperature".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);

        let topic_name = r"/sensor/1/2/temperature3".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), false);

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"/sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), false);

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"/sensor/+".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);

        let topic_name = r"/sensor/temperature3/tmpq".to_string();
        let sub_regex = r"/sensor/#".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);

        let topic_name = "/topic/test".to_string();
        let sub_regex = "$share/groupname/topic/test".to_string();
        assert!(path_regex_match(topic_name, sub_regex));

        let topic_name = r"/sensor/1/temperature".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);

        let topic_name = r"/sensor/1/2/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), false);

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+/temperature".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), false);

        let topic_name = r"/sensor/temperature3".to_string();
        let sub_regex = r"$share/groupname/sensor/+".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);

        let topic_name = r"/sensor/temperature3/tmpq".to_string();
        let sub_regex = r"$share/groupname/sensor/#".to_string();
        assert_eq!(path_regex_match(topic_name, sub_regex), true);
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
        let topic = MQTTTopic::new(&topic_name);
        metadata_cache.add_topic(&topic_name, &topic);

        let sub_path = "/test/topic".to_string();
        let result = get_sub_topic_id_list(metadata_cache.clone(), sub_path).await;
        assert!(result.len() == 1);
        assert_eq!(result.get(0).unwrap().clone(), topic.topic_id);
    }

    #[tokio::test]
    async fn path_validator_test() {
        let path = "/loboxu/test".to_string();
        assert!(sub_path_validator(path));

        let path = "/loboxu/#".to_string();
        assert!(sub_path_validator(path));

        let path = "/loboxu/+".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/#".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/#/test".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/+/test".to_string();
        assert!(sub_path_validator(path));

        let path = "$share/loboxu/+test".to_string();
        assert!(!sub_path_validator(path));

        let path = "$share/loboxu/#test".to_string();
        assert!(!sub_path_validator(path));

        let path = "$share/loboxu/*test".to_string();
        assert!(!sub_path_validator(path));
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

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_poll.clone());
        let new_sub = true;
        let dup_msg = true;

        let topic_name = "/test/topic".to_string();
        let payload = "testtesttest".to_string();
        let topic = MQTTTopic::new(&topic_name);

        metadata_cache.add_topic(&topic_name, &topic);

        let mut retain_message = MQTTMessage::default();
        retain_message.dup = false;
        retain_message.qos = QoS::AtLeastOnce;
        retain_message.pkid = 1;
        retain_message.retain = true;
        retain_message.topic = Bytes::from(topic_name.clone());
        retain_message.payload = Bytes::from(payload);

        match topic_storage
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
            client_poll,
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
