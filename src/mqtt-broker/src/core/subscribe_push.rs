use crate::core::metadata_cache::MetadataCacheManager;
use crate::{
    core::subscribe::max_qos,
    core::subscribe_share::share_sub_rewrite_publish_flag,
    metadata::message::Message,
    server::{tcp::packet::ResponsePackage, MQTTProtocol},
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::log::{error, info};
use dashmap::DashMap;
use protocol::mqtt::{MQTTPacket, Publish, PublishProperties};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    sync::broadcast::{self, Sender},
    time::sleep,
};

pub struct PushServer<S> {
    metadata_cache: Arc<MetadataCacheManager>,
    topic_push_thread: DashMap<String, Sender<bool>>,
    message_storage_adapter: Arc<S>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
}

impl<S> PushServer<S>
where
    S: StorageAdapter + Send + Sync + 'static,
{
    pub fn new(
        metadata_cache: Arc<MetadataCacheManager>,
        message_storage_adapter: Arc<S>,
        response_queue_sx4: Sender<ResponsePackage>,
        response_queue_sx5: Sender<ResponsePackage>,
    ) -> Self {
        return PushServer {
            metadata_cache,
            topic_push_thread: DashMap::with_capacity(256),
            message_storage_adapter,
            response_queue_sx4,
            response_queue_sx5,
        };
    }

    pub async fn start(&self) {
        info("Subscription push thread is started successfully.".to_string());
        loop {
            self.parse_filter();
            self.push_thread();
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn parse_filter(&self) {
        let sub_identifier = if let Some(properties) = subscribe_properties {
            properties.subscription_identifier
        } else {
            None
        };

        for (topic_id, topic_name) in self.metadata_cache.topic_id_name.clone() {
            if !self.topic_subscribe.contains_key(&topic_id) {
                self.topic_subscribe
                    .insert(topic_id.clone(), DashMap::with_capacity(256));
            }

            if !self.client_subscribe.contains_key(&client_id) {
                self.client_subscribe
                    .insert(client_id.clone(), DashMap::with_capacity(256));
            }

            let tp_sub = self.topic_subscribe.get_mut(&topic_id).unwrap();
            let client_sub = self.client_subscribe.get_mut(&client_id).unwrap();
            for filter in subscribe.filters.clone() {
                if is_share_sub(filter.path.clone()) {
                    let (group_name, sub_name) = decode_share_info(filter.path.clone());
                    if path_regex_match(topic_name.clone(), sub_name.clone()) {
                        let conf = broker_mqtt_conf();
                        let req = GetShareSubRequest {
                            cluster_name: conf.cluster_name.clone(),
                            group_name,
                            sub_name: sub_name.clone(),
                        };
                        match placement_get_share_sub(
                            client_poll.clone(),
                            conf.placement.server.clone(),
                            req,
                        )
                        .await
                        {
                            Ok(reply) => {
                                info(format!(
                                    " Leader node for the shared subscription is [{}]",
                                    reply.broker_id
                                ));
                                if reply.broker_id != conf.broker_id {
                                    //todo
                                } else {
                                    let sub = Subscriber {
                                        protocol: protocol.clone(),
                                        client_id: client_id.clone(),
                                        packet_identifier: subscribe.packet_identifier,
                                        qos: filter.qos,
                                        nolocal: filter.nolocal,
                                        preserve_retain: filter.preserve_retain,
                                        subscription_identifier: sub_identifier,
                                        user_properties: Vec::new(),
                                        is_share_sub: true,
                                    };
                                    tp_sub.insert(client_id.clone(), sub);
                                    client_sub.insert(topic_id.clone(), now_second());
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                } else {
                    if path_regex_match(topic_name.clone(), filter.path.clone()) {
                        let sub = Subscriber {
                            protocol: protocol.clone(),
                            client_id: client_id.clone(),
                            packet_identifier: subscribe.packet_identifier,
                            qos: filter.qos,
                            nolocal: filter.nolocal,
                            preserve_retain: filter.preserve_retain,
                            subscription_identifier: sub_identifier,
                            user_properties: Vec::new(),
                            is_share_sub: false,
                        };
                        tp_sub.insert(client_id.clone(), sub);
                        client_sub.insert(topic_id.clone(), now_second());
                    }
                }
            }
        }
    }

    pub fn push_thread(&self) {
        for (topic_id, list) in self.subscribe_manager.topic_subscribe.clone() {
            // If the topic has no subscribers,
            // remove the topic information from the subscription relationship cache and stop the topic push management thread.
            if list.len() == 0 {
                if let Some(sx) = self.topic_push_thread.get(&topic_id) {
                    match sx.send(true) {
                        Ok(_) => {
                            info(format!(
                                "Push thread for Topic [{}] was stopped successfully",
                                topic_id
                            ));
                        }
                        Err(e) => {
                            error(e.to_string());
                        }
                    }
                }

                self.subscribe_manager.remove_topic(topic_id.clone());
                continue;
            }

            // 1. If no push thread is detected for topic, the corresponding thread is created for topic dimension push management.
            if !self.topic_push_thread.contains_key(&topic_id) {
                let (sx, mut rx) = broadcast::channel(1000);
                let response_queue_sx4 = self.response_queue_sx4.clone();
                let response_queue_sx5 = self.response_queue_sx5.clone();
                let storage_adapter = self.message_storage_adapter.clone();
                let subscribe_manager = self.subscribe_manager.clone();
                let metadata_cache = self.metadata_cache.clone();
                self.topic_push_thread.insert(topic_id.clone(), sx);

                tokio::spawn(async move {
                    info(format!(
                        "Push thread for Topic [{}] was started successfully",
                        topic_id
                    ));
                    loop {
                        match rx.try_recv() {
                            Ok(flag) => {
                                if flag {
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                        let message_storage = MessageStorage::new(storage_adapter.clone());

                        topic_sub_push_thread(
                            metadata_cache.clone(),
                            message_storage,
                            topic_id.clone(),
                            response_queue_sx4.clone(),
                            response_queue_sx5.clone(),
                        )
                        .await;
                    }
                });
            }
        }
    }
}

pub async fn topic_sub_push_thread<T, S>(
    metadata_cache: Arc<MetadataCacheManager>,
    message_storage: MessageStorage<S>,
    topic_id: String,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
) where
    S: StorageAdapter + StorageAdapter + Send + Sync + 'static,
{
    let group_id = format!("system_sub_{}", topic_id);
    let record_num = 5;
    let max_wait_ms = 500;
    loop {
        let topic_sub = subscribe_manager.topic_subscribe.clone();
        for (topic_name, sub_list) in topic_sub {
            if sub_list.len() == 0 {
                sleep(Duration::from_millis(max_wait_ms)).await;
                continue;
            }
            match message_storage
                .read_topic_message(topic_id.clone(), group_id.clone(), record_num)
                .await
            {
                Ok(result) => {
                    if result.len() == 0 {
                        sleep(Duration::from_millis(max_wait_ms)).await;
                        continue;
                    }
                    // commit offset
                    if let Some(last_res) = result.last() {
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
                                continue;
                            }
                        }
                    }

                    // Push data to subscribers
                    for (_, subscribe) in sub_list {
                        let mut sub_id = Vec::new();
                        if let Some(id) = subscribe.subscription_identifier {
                            sub_id.push(id);
                        }

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
                        for record in result.clone() {
                            let msg = match Message::decode_record(record) {
                                Ok(msg) => msg,
                                Err(e) => {
                                    error(e.to_string());
                                    continue;
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
                            if subscribe.is_share_sub {
                                user_properteis.push(share_sub_rewrite_publish_flag());
                            }

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
                    }
                }
                Err(e) => {
                    error(e.to_string());
                    sleep(Duration::from_millis(max_wait_ms)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::metadata::message::Message;
    use crate::subscribe::push::topic_sub_push_thread;
    use crate::{
        metadata::{cache::MetadataCacheManager, topic::Topic},
        storage::message::MessageStorage,
    };
    use bytes::Bytes;
    use clients::poll::ClientPool;
    use protocol::mqtt::{Filter, MQTTPacket, Subscribe};
    use std::sync::Arc;
    use storage_adapter::memory::MemoryStorageAdapter;
    use storage_adapter::record::Record;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn topic_sub_push_thread_test() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(MetadataCacheManager::new("test-cluster".to_string()));

        let client_poll = Arc::new(ClientPool::new(3));

        // Create topic
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        metadata_cache.set_topic(&topic_name, &topic);

        // Subscription topic
        let client_id = "test-ttt".to_string();
        let packet_identifier = 2;
        let mut filters = Vec::new();
        let filter = Filter {
            path: "/test/topic".to_string(),
            qos: protocol::mqtt::QoS::AtLeastOnce,
            nolocal: true,
            preserve_retain: true,
            retain_forward_rule: protocol::mqtt::RetainForwardRule::Never,
        };
        filters.push(filter);
        let subscribe = Subscribe {
            packet_identifier,
            filters,
        };
        sub_manager
            .parse_subscribe(
                crate::server::MQTTProtocol::MQTT5,
                client_id,
                subscribe,
                None,
                client_poll.clone(),
            )
            .await;

        // Start push thread
        let message_storage = MessageStorage::new(storage_adapter.clone());
        let (response_queue_sx4, mut response_queue_rx4) = broadcast::channel(1000);
        let (response_queue_sx5, mut response_queue_rx5) = broadcast::channel(1000);
        let ms = message_storage.clone();
        let topic_id: String = topic.topic_id.clone();
        tokio::spawn(async move {
            topic_sub_push_thread(
                metadata_cache,
                ms,
                topic_id,
                response_queue_sx4,
                response_queue_sx5,
            )
            .await;
        });

        // Send data
        let mut msg = Message::default();
        msg.payload = Bytes::from("testtest".to_string());

        let record = Record::build_b(serde_json::to_vec(&msg).unwrap());
        message_storage
            .append_topic_message(topic.topic_id.clone(), vec![record])
            .await
            .unwrap();

        // Receive subscription data
        loop {
            match response_queue_rx5.recv().await {
                Ok(packet) => {
                    if let MQTTPacket::Publish(publish, _) = packet.packet {
                        assert_eq!(publish.topic, topic.topic_id);
                        assert_eq!(publish.payload, msg.payload);
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
