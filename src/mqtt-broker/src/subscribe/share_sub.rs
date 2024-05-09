use super::{manager::SubscribeManager, subscribe::max_qos};
use crate::{
    core::metadata_cache::MetadataCacheManager,
    metadata::{message::Message, subscriber::Subscriber},
    server::{tcp::packet::ResponsePackage, MQTTProtocol},
    storage::message::MessageStorage,
    subscribe::manager::ShareSubShareSub,
};

use bytes::Bytes;
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info},
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use protocol::{
    mqtt::{MQTTPacket, Publish, PublishProperties, SubscribeProperties},
    mqttv4::codec::Mqtt4Codec,
    mqttv5::codec::Mqtt5Codec,
};
use std::{sync::Arc, time::Duration};
use storage_adapter::{record::Record, storage::StorageAdapter};
use tokio::{
    net::TcpStream,
    sync::{
        broadcast,
        mpsc::{self, Receiver, Sender},
    },
    time::sleep,
};
use tokio_util::codec::Framed;

const SHARE_SUB_PREFIX: &str = "$share";
const SHARE_SUB_REWRITE_PUBLISH_FLAG: &str = "$system_ssrpf";
const SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE: &str = "True";

const SHARED_SUBSCRIPTION_STRATEGY_ROUND_ROBIN: &str = "round_robin";
const SHARED_SUBSCRIPTION_STRATEGY_RANDOM: &str = "random";
const SHARED_SUBSCRIPTION_STRATEGY_STICKY: &str = "sticky";
const SHARED_SUBSCRIPTION_STRATEGY_HASH: &str = "hash";
const SHARED_SUBSCRIPTION_STRATEGY_LOCAL: &str = "local";

#[derive(Clone)]
pub struct SubscribeShare<S> {
    // (topic_id, Sender<bool>)
    pub leader_pull_data_thread: DashMap<String, Sender<bool>>,

    // (topic_id, Sender<bool>)
    pub leader_push_data_thread: DashMap<String, Sender<bool>>,

    // (topic_id, Sender<bool>)
    pub follower_sub_thread: DashMap<String, Sender<bool>>,

    pub subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
    message_storage: Arc<S>,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
    metadata_cache: Arc<MetadataCacheManager>,
}

impl<S> SubscribeShare<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        client_poll: Arc<ClientPool>,
        subscribe_manager: Arc<SubscribeManager>,
        message_storage: Arc<S>,
        response_queue_sx4: broadcast::Sender<ResponsePackage>,
        response_queue_sx5: broadcast::Sender<ResponsePackage>,
        metadata_cache: Arc<MetadataCacheManager>,
    ) -> Self {
        return SubscribeShare {
            leader_pull_data_thread: DashMap::with_capacity(128),
            leader_push_data_thread: DashMap::with_capacity(128),
            follower_sub_thread: DashMap::with_capacity(128),
            subscribe_manager,
            client_poll,
            message_storage,
            response_queue_sx4,
            response_queue_sx5,
            metadata_cache,
        };
    }

    pub async fn start_leader_push(&self) {
        let conf = broker_mqtt_conf();
        loop {
            for (topic_id, sub_list) in self.subscribe_manager.share_leader_subscribe.clone() {
                if sub_list.len() == 0 {
                    // stop pull data thread
                    if let Some(sx) = self.leader_pull_data_thread.get(&topic_id) {
                        match sx.send(true).await {
                            Ok(_) => {
                                self.leader_pull_data_thread.remove(&topic_id);
                            }
                            Err(e) => error(e.to_string()),
                        }
                    }

                    // stop push data thread
                    if let Some(sx) = self.leader_push_data_thread.get(&topic_id) {
                        match sx.send(true).await {
                            Ok(_) => {
                                self.leader_push_data_thread.remove(&topic_id);
                            }
                            Err(e) => error(e.to_string()),
                        }
                    }

                    self.subscribe_manager
                        .share_leader_subscribe
                        .remove(&topic_id);
                    continue;
                }

                let (sx, rx) = mpsc::channel(10000);

                let subscribe_manager = self.subscribe_manager.clone();
                // start pull data thread
                if !self.leader_pull_data_thread.contains_key(&topic_id) {
                    self.start_topic_pull_data_thread(topic_id.clone(), sx)
                        .await;
                }

                // start push data thread
                if !self.leader_push_data_thread.contains_key(&topic_id) {
                    // round_robin
                    if conf.subscribe.shared_subscription_strategy
                        == SHARED_SUBSCRIPTION_STRATEGY_ROUND_ROBIN.to_string()
                    {
                        self.start_push_by_round_robin(topic_id.clone(), rx, subscribe_manager);
                    }

                    // random
                    if conf.subscribe.shared_subscription_strategy
                        == SHARED_SUBSCRIPTION_STRATEGY_RANDOM.to_string()
                    {
                        self.start_push_by_random();
                    }

                    // sticky
                    if conf.subscribe.shared_subscription_strategy
                        == SHARED_SUBSCRIPTION_STRATEGY_STICKY.to_string()
                    {
                        self.start_push_by_sticky();
                    }

                    // hash
                    if conf.subscribe.shared_subscription_strategy
                        == SHARED_SUBSCRIPTION_STRATEGY_HASH.to_string()
                    {
                        self.start_push_by_hash();
                    }

                    // local
                    if conf.subscribe.shared_subscription_strategy
                        == SHARED_SUBSCRIPTION_STRATEGY_LOCAL.to_string()
                    {
                        self.start_push_by_local();
                    }
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn start_topic_pull_data_thread(&self, topic_id: String, channel_sx: Sender<Record>) {
        let (sx, mut rx) = mpsc::channel(1);
        self.leader_pull_data_thread.insert(topic_id.clone(), sx);
        let message_storage = self.message_storage.clone();
        tokio::spawn(async move {
            info(format!(
                "Share push thread for Topic [{}] was started successfully",
                topic_id
            ));
            let message_storage = MessageStorage::new(message_storage);
            let group_id = format!("system_sub_{}", topic_id);
            let record_num = 100;
            let max_wait_ms = 500;
            loop {
                match rx.try_recv() {
                    Ok(flag) => {
                        if flag {
                            info(format!(
                                "Exclusive Push thread for Topic [{}] was stopped successfully",
                                topic_id
                            ));
                            break;
                        }
                    }
                    Err(_) => {}
                }

                match message_storage
                    .read_topic_message(topic_id.clone(), group_id.clone(), record_num as u128)
                    .await
                {
                    Ok(results) => {
                        if results.len() == 0 {
                            sleep(Duration::from_millis(max_wait_ms)).await;
                            return;
                        }

                        // commit offset
                        if let Some(last_res) = results.last() {
                            match message_storage
                                .commit_group_offset(
                                    topic_id.clone(),
                                    group_id.clone(),
                                    last_res.offset,
                                )
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    error(e.to_string());
                                    return;
                                }
                            }
                        }

                        // Push data to subscribers
                        for record in results.clone() {
                            match channel_sx.send(record).await {
                                Ok(_) => {}
                                Err(e) => error(e.to_string()),
                            }
                        }
                    }
                    Err(e) => {
                        error(e.to_string());
                        sleep(Duration::from_millis(max_wait_ms)).await;
                    }
                }
            }
        });
    }

    pub fn start_push_by_round_robin(
        &self,
        topic_id: String,
        mut channel_rx: Receiver<Record>,
        subscribe_manager: Arc<SubscribeManager>,
    ) {
        let (sx, mut rx) = mpsc::channel(1);
        self.leader_push_data_thread.insert(topic_id.clone(), sx);
        let response_queue_sx4 = self.response_queue_sx4.clone();
        let response_queue_sx5 = self.response_queue_sx5.clone();
        let metadata_cache = self.metadata_cache.clone();

        tokio::spawn(async move {
            info(format!(
                "Share push thread for Topic [{}] was started successfully",
                topic_id
            ));

            loop {
                match rx.try_recv() {
                    Ok(flag) => {
                        if flag {
                            info(format!(
                                "Exclusive Push thread for Topic [{}] was stopped successfully",
                                topic_id
                            ));
                            break;
                        }
                    }
                    Err(_) => {}
                }

                if let Some(sub_list) = subscribe_manager.share_leader_subscribe.get(&topic_id) {
                    for (_, subscribe) in sub_list.clone() {
                        let connect_id = if let Some(sess) =
                            metadata_cache.session_info.get(&subscribe.client_id)
                        {
                            if let Some(conn_id) = sess.connection_id {
                                conn_id
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };

                        let topic_name =
                            if let Some(topic_name) = metadata_cache.topic_id_name.get(&topic_id) {
                                topic_name.clone()
                            } else {
                                continue;
                            };
                        if let Some(record) = channel_rx.recv().await {
                            publish_to_client(
                                connect_id,
                                topic_name,
                                subscribe,
                                record,
                                response_queue_sx4.clone(),
                                response_queue_sx5.clone(),
                            )
                            .await;
                        }
                    }
                }
            }
        });
    }

    fn start_push_by_random(&self) {}

    fn start_push_by_hash(&self) {}

    fn start_push_by_sticky(&self) {}

    fn start_push_by_local(&self) {}

    pub async fn start_follower_sub(&self) {
        for (client_id, share_sub) in self.subscribe_manager.share_follower_subscribe.clone() {
            let (sx, rx) = mpsc::channel(1);
            self.leader_pull_data_thread.insert(client_id.clone(), sx);
            tokio::spawn(async move {
                if share_sub.protocol == MQTTProtocol::MQTT4 {
                    rewrite_sub_mqtt4(share_sub, rx).await;
                } else if share_sub.protocol == MQTTProtocol::MQTT5 {
                    rewrite_sub_mqtt5(share_sub, rx).await;
                }
            });
        }
    }
}

async fn rewrite_sub_mqtt4(share_sub: ShareSubShareSub, mut rx: Receiver<bool>) {
    let socket = TcpStream::connect(share_sub.leader_addr.clone())
        .await
        .unwrap();
    let stream: Framed<TcpStream, Mqtt4Codec> = Framed::new(socket, Mqtt4Codec::new());
}

async fn rewrite_sub_mqtt5(share_sub: ShareSubShareSub, mut rx: Receiver<bool>) {
    let socket = TcpStream::connect(share_sub.leader_addr.clone())
        .await
        .unwrap();
    let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());
    let packet = build_rewrite_subscribe_pkg(share_sub);
    let _ = stream.send(packet).await;
    loop {
        if let Some(data) = stream.next().await {
            match data {
                Ok(da) => match da {
                    MQTTPacket::Publish(publish, publish_properties) => {
                        
                    }

                    MQTTPacket::PubRec(publish, publish_properties) => {}

                    MQTTPacket::PubComp(publish, publish_properties) => {}

                    MQTTPacket::Disconnect(publish, publish_properties) => {}
                    _ => {
                        error("".to_string());
                    }
                },
                Err(e) => error(e.to_string()),
            }
        }
        match rx.try_recv() {
            Ok(flag) => {
                if flag {
                    break;
                }
            }
            Err(_) => {}
        }
    }
}

async fn publish_to_client(
    connect_id: u64,
    topic_name: String,
    subscribe: Subscriber,
    record: Record,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) {
    let mut sub_id = Vec::new();
    if let Some(id) = subscribe.subscription_identifier {
        sub_id.push(id);
    }

    let msg = match Message::decode_record(record) {
        Ok(msg) => msg,
        Err(e) => {
            error(e.to_string());
            return;
        }
    };

    let publish = Publish {
        dup: false,
        qos: max_qos(msg.qos, subscribe.qos),
        pkid: subscribe.packet_identifier,
        retain: false,
        topic: Bytes::from(topic_name.clone()),
        payload: Bytes::from(msg.payload),
    };

    // If it is a shared subscription, it will be identified with the push message
    let mut user_properteis = Vec::new();
    user_properteis.push(share_sub_rewrite_publish_flag());

    let properties = PublishProperties {
        payload_format_indicator: None,
        message_expiry_interval: None,
        topic_alias: None,
        response_topic: None,
        correlation_data: None,
        user_properties: user_properteis,
        subscription_identifiers: sub_id.clone(),
        content_type: None,
    };

    let resp = ResponsePackage {
        connection_id: connect_id,
        packet: MQTTPacket::Publish(publish, Some(properties)),
    };

    if subscribe.protocol == MQTTProtocol::MQTT4 {
        match response_queue_sx4.send(resp) {
            Ok(_) => {}
            Err(e) => error(format!("{}", e.to_string())),
        }
    } else if subscribe.protocol == MQTTProtocol::MQTT5 {
        match response_queue_sx5.send(resp) {
            Ok(_) => {}
            Err(e) => error(format!("{}", e.to_string())),
        }
    }
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

pub fn is_share_sub_rewrite_publish(publish_properties: Option<PublishProperties>) -> bool {
    if let Some(properties) = publish_properties {
        for (k, v) in properties.user_properties {
            if k == SHARE_SUB_REWRITE_PUBLISH_FLAG
                && v == SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE.to_string()
            {
                return true;
            }
        }
    }
    return false;
}

pub fn build_rewrite_subscribe_pkg(rewrite_sub: ShareSubShareSub) -> MQTTPacket {
    let subscribe = rewrite_sub.subscribe.clone();
    let mut subscribe_properties =
        if let Some(properties) = rewrite_sub.subscribe_properties.clone() {
            properties
        } else {
            SubscribeProperties::default()
        };

    let mut user_properties = Vec::new();
    user_properties.push(share_sub_rewrite_publish_flag());
    subscribe_properties.user_properties = user_properties;
    return MQTTPacket::Subscribe(subscribe, Some(subscribe_properties));
}

pub fn share_sub_rewrite_publish_flag() -> (String, String) {
    return (
        SHARE_SUB_REWRITE_PUBLISH_FLAG.to_string(),
        SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE.to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::decode_share_info;
    use crate::subscribe::share_sub::is_share_sub;

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
}
