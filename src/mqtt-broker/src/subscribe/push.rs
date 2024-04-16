use super::manager::SubScribeManager;
use crate::{
    handler::subscribe::max_qos, metadata::message::Message, server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::log::{error, info};
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
    response_queue_sx: Sender<ResponsePackage>,
}

impl PushServer {
    pub fn new(
        subscribe_manager: Arc<RwLock<SubScribeManager>>,
        storage_adapter: Arc<MemoryStorageAdapter>,
        response_queue_sx: Sender<ResponsePackage>,
    ) -> Self {
        return PushServer {
            subscribe_manager,
            topic_push_thread: HashMap::new(),
            storage_adapter,
            response_queue_sx,
        };
    }

    pub async fn start(&mut self) {
        info("Subscription push thread is started successfully.".to_string());
        loop {
            let sub_manager = self.subscribe_manager.read().await;
            let topic_subscribe = sub_manager.topic_subscribe.clone();
            drop(sub_manager);

            for (topic_id, list) in topic_subscribe {
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

#[cfg(test)]
mod tests {
    use crate::metadata::message::Message;
    use crate::subscribe::push::topic_sub_push_thread;
    use crate::{
        metadata::{cache::MetadataCache, topic::Topic},
        storage::message::MessageStorage,
        subscribe::manager::SubScribeManager,
    };
    use bytes::Bytes;
    use protocol::mqtt::{Filter, MQTTPacket, Subscribe};
    use std::sync::Arc;
    use storage_adapter::memory::MemoryStorageAdapter;
    use storage_adapter::record::Record;
    use tokio::sync::{broadcast, RwLock};

    #[tokio::test]
    async fn topic_sub_push_thread_test() {
        let storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let metadata_cache = Arc::new(RwLock::new(MetadataCache::new(storage_adapter.clone())));

        // Create topic
        let topic_name = "/test/topic".to_string();
        let topic = Topic::new(&topic_name);
        let mut cache = metadata_cache.write().await;
        cache.set_topic(&topic_name, &topic);
        drop(cache);

        let sub_manager = Arc::new(RwLock::new(SubScribeManager::new(metadata_cache)));

        // Subscription topic
        let mut sub_m = sub_manager.write().await;
        let connect_id = 1;
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
        sub_m.parse_subscribe(connect_id, subscribe, None).await;
        drop(sub_m);

        // Start push thread
        let message_storage = MessageStorage::new(storage_adapter.clone());
        let (response_queue_sx, mut response_queue_rx) = broadcast::channel(1000);
        let ms = message_storage.clone();
        let topic_id: String = topic.topic_id.clone();
        tokio::spawn(async move {
            topic_sub_push_thread(sub_manager, ms, topic_id, response_queue_sx).await;
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
            match response_queue_rx.recv().await {
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
