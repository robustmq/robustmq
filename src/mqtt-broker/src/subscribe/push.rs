use super::manager::SubScribeManager;
use crate::{
    handler::subscribe::max_qos,
    metadata::{cache::MetadataCache, message::Message},
    server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::log::error;
use protocol::mqtt::{MQTTPacket, Publish, PublishProperties};
use std::{collections::HashMap, sync::Arc, time::Duration};
use storage_adapter::memory::MemoryStorageAdapter;
use tokio::{
    sync::{
        broadcast::{self, Sender},
        RwLock,
    },
    time::sleep,
};

pub struct PushServer {
    subscribe_manager: Arc<RwLock<SubScribeManager>>,
    topic_push_thread: HashMap<String, Sender<bool>>,
    storage_adapter: Arc<MemoryStorageAdapter>,
    metadata_cache: Arc<RwLock<MetadataCache>>,
    response_queue_sx: Sender<ResponsePackage>,
}

impl PushServer {
    pub fn new(
        subscribe_manager: Arc<RwLock<SubScribeManager>>,
        storage_adapter: Arc<MemoryStorageAdapter>,
        metadata_cache: Arc<RwLock<MetadataCache>>,
        response_queue_sx: Sender<ResponsePackage>,
    ) -> Self {
        return PushServer {
            subscribe_manager,
            topic_push_thread: HashMap::new(),
            storage_adapter,
            metadata_cache,
            response_queue_sx,
        };
    }

    pub async fn start(&mut self) {
        loop {
            let sub_manager = self.subscribe_manager.read().await;
            let topic_subscribe_num = sub_manager.topic_subscribe_num.clone();
            drop(sub_manager);

            for (topic_id, sub_num) in topic_subscribe_num {
                // If the topic has no subscribers,
                // remove the topic information from the subscription relationship cache and stop the topic push management thread.
                if sub_num == 0 {
                    if let Some(sx) = self.topic_push_thread.get(&topic_id) {
                        match sx.send(true) {
                            Ok(_) => {}
                            Err(e) => {
                                error(e.to_string());
                            }
                        }
                    }
                    let mut sub_manager_w = self.subscribe_manager.write().await;
                    sub_manager_w.remove_topic(topic_id.clone());
                    continue;
                }

                // 1. If no push thread is detected for topic, the corresponding thread is created for topic dimension push management.
                if !self.topic_push_thread.contains_key(&topic_id) {
                    let (sx, mut rx) = broadcast::channel(1000);
                    let response_queue_sx = self.response_queue_sx.clone();
                    let storage_adapter = self.storage_adapter.clone();
                    let subscribe_manager = self.subscribe_manager.clone();
                    self.topic_push_thread.insert(topic_id.clone(), sx);

                    tokio::spawn(async move {
                        loop {
                            match rx.recv().await {
                                Ok(flag) => {
                                    if flag {
                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                            let message_storage = MessageStorage::new(storage_adapter.clone());

                            topic_sub_push_thread(
                                subscribe_manager.clone(),
                                message_storage,
                                topic_id.clone(),
                                response_queue_sx.clone(),
                            )
                            .await;
                        }
                    });
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}

pub async fn topic_sub_push_thread(
    subscribe_manager: Arc<RwLock<SubScribeManager>>,
    message_storage: MessageStorage,
    topic_id: String,
    response_queue_sx: Sender<ResponsePackage>,
) {
    let group_id = format!("system_sub_{}", topic_id);
    let record_num = 5;
    let max_wait_ms = 500;
    loop {
        let sub_manager = subscribe_manager.read().await;
        let topic_sub = sub_manager.topic_subscribe.clone();
        drop(sub_manager);
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
                    for subscribe in sub_list {
                        let mut sub_id = Vec::new();
                        if let Some(id) = subscribe.subscription_identifier {
                            sub_id.push(id);
                        }
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
                                connection_id: subscribe.connect_id,
                                packet: MQTTPacket::Publish(publish, Some(properties)),
                            };
                            match response_queue_sx.send(resp) {
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
}
