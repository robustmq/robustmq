use crate::core::cache_manager::CacheManager;
use crate::core::cache_manager::QosAckPackageData;
use crate::{server::tcp::packet::ResponsePackage, storage::message::MessageStorage};
use clients::placement::mqtt::call::placement_get_share_sub_leader;
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::{errors::RobustMQError, log::error};
use protocol::mqtt::common::{MQTTPacket, PubRel, Publish, PublishProperties, QoS};
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
        let (_, group_path) = decode_share_info(sub_path);
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

pub fn min_qos(qos: QoS, sub_qos: QoS) -> QoS {
    if qos <= sub_qos {
        return qos;
    }
    return sub_qos;
}

pub async fn get_sub_topic_id_list(
    metadata_cache: Arc<CacheManager>,
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
    resp: ResponsePackage,
    response_queue_sx: broadcast::Sender<ResponsePackage>,
) -> Result<(), RobustMQError> {
    match response_queue_sx.send(resp) {
        Ok(_) => {}
        Err(e) => return Err(RobustMQError::CommmonError(format!("{}", e.to_string()))),
    }

    return Ok(());
}

pub async fn qos2_send_publish(
    metadata_cache: Arc<CacheManager>,
    client_id: String,
    mut publish: Publish,
    publish_properties: Option<PublishProperties>,
    response_queue_sx: Sender<ResponsePackage>,
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
                resp.clone(),
                response_queue_sx.clone(),
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
    metadata_cache: Arc<CacheManager>,
    client_id: String,
    pkid: u16,
    response_queue_sx: Sender<ResponsePackage>,
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
            reason: protocol::mqtt::common::PubRelReason::Success,
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
                pubrel_resp.clone(),
                response_queue_sx.clone(),
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
    metadata_cache: Arc<CacheManager>,
    mqtt_client_id: String,
    publish: Publish,
    response_queue_sx: Sender<ResponsePackage>,
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
    match publish_to_response_queue(resp.clone(), response_queue_sx.clone()).await {
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
    use crate::core::cache_manager::CacheManager;
    use crate::subscribe::sub_common::{decode_share_info, is_share_sub, sub_path_validator};
    use crate::subscribe::sub_common::{get_sub_topic_id_list, min_qos, path_regex_match};
    use clients::poll::ClientPool;
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::QoS;
    use std::sync::Arc;
    use storage_adapter::memory::MemoryStorageAdapter;

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
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(100));
        let metadata_cache = Arc::new(CacheManager::new(
            client_poll,
            "test-cluster".to_string(),
            4,
        ));
        let topic_name = "/test/topic".to_string();
        let topic = MQTTTopic::new(unique_id(), topic_name.clone());
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
}
