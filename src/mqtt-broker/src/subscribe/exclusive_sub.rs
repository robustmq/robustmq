use crate::{
    core::metadata_cache::MetadataCacheManager,
    metadata::{message::Message, subscriber::Subscriber},
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

use super::{sub_manager::SubscribeManager, subscribe::max_qos};
#[derive(Clone)]
pub struct SubscribeExclusive<S> {
    metadata_cache: Arc<MetadataCacheManager>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    subscribe_manager: Arc<SubscribeManager>,
    message_storage: Arc<S>,
    // (topic_id, Sender<bool>)
    exclusive_sub_push_thread: DashMap<String, Sender<bool>>,
}

impl<S> SubscribeExclusive<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        message_storage: Arc<S>,
        metadata_cache: Arc<MetadataCacheManager>,
        response_queue_sx4: Sender<ResponsePackage>,
        response_queue_sx5: Sender<ResponsePackage>,
        subscribe_manager: Arc<SubscribeManager>,
    ) -> Self {
        return SubscribeExclusive {
            message_storage,
            metadata_cache,
            response_queue_sx4,
            response_queue_sx5,
            exclusive_sub_push_thread: DashMap::with_capacity(256),
            subscribe_manager,
        };
    }

    pub async fn start(&self) {
        loop {
            self.exclusive_sub_push_thread();
            sleep(Duration::from_secs(1)).await;
        }
    }

    // Handles exclusive subscription push tasks
    // Exclusively subscribed messages are pushed directly to the consuming client
    fn exclusive_sub_push_thread(&self) {
        for (topic_id, list) in self.subscribe_manager.exclusive_subscribe.clone() {
            // If the topic has no subscribers,
            // remove the topic information from the subscription relationship cache and stop the topic push management thread.
            if list.len() == 0 {
                if let Some(sx) = self.exclusive_sub_push_thread.get(&topic_id) {
                    match sx.send(true) {
                        Ok(_) => {
                            self.exclusive_sub_push_thread.remove(&topic_id);
                        }
                        Err(e) => {
                            error(e.to_string());
                        }
                    }
                }
                continue;
            }

            // 1. If no push thread is detected for topic, the corresponding thread is created for topic dimension push management.
            if !self.exclusive_sub_push_thread.contains_key(&topic_id) {
                let (sx, mut rx) = broadcast::channel(2);
                let response_queue_sx4 = self.response_queue_sx4.clone();
                let response_queue_sx5 = self.response_queue_sx5.clone();
                let metadata_cache = self.metadata_cache.clone();
                let subscribe_manager = self.subscribe_manager.clone();
                let message_storage = self.message_storage.clone();

                // Subscribe to the data push thread
                self.exclusive_sub_push_thread.insert(topic_id.clone(), sx);
                tokio::spawn(async move {
                    info(format!(
                        "Exclusive push thread for Topic [{}] was started successfully",
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
                        // let message_storage = MessageStorage::new(storage_adapter.clone());

                        if let Some(sub_list) = subscribe_manager.exclusive_subscribe.get(&topic_id)
                        {
                            push_thread(
                                sub_list.clone(),
                                metadata_cache.clone(),
                                message_storage.clone(),
                                topic_id.clone(),
                                response_queue_sx4.clone(),
                                response_queue_sx5.clone(),
                            )
                            .await;
                        } else {
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                });
            }
        }
    }
}

async fn push_thread<S>(
    sub_list: DashMap<String, Subscriber>,
    metadata_cache: Arc<MetadataCacheManager>,
    message_storage: Arc<S>,
    topic_id: String,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let message_storage = MessageStorage::new(message_storage);
    let group_id = format!("system_sub_{}", topic_id);
    let record_num = 5;
    let max_wait_ms = 100;

    for (topic_name, subscribe) in sub_list {
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
                        .commit_group_offset(topic_id.clone(), group_id.clone(), last_res.offset)
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
                let mut sub_id = Vec::new();
                if let Some(id) = subscribe.subscription_identifier {
                    sub_id.push(id);
                }

                let connect_id =
                    if let Some(sess) = metadata_cache.session_info.get(&subscribe.client_id) {
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

                    let user_properteis = Vec::new();

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
            Err(e) => {
                error(e.to_string());
                sleep(Duration::from_millis(max_wait_ms)).await;
            }
        }
    }
}
