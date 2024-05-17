use crate::{
    core::metadata_cache::MetadataCacheManager,
    metadata::message::Message,
    qos::ack_manager::{AckManager, AckPacketInfo},
    server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::{
    log::{error, info},
    tools::now_second,
};
use dashmap::DashMap;
use protocol::mqtt::{MQTTPacket, Publish, PublishProperties};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    sync::{
        broadcast::{self, Sender},
        mpsc,
    },
    time::{sleep, timeout},
};

use super::{
    sub_manager::SubscribeManager,
    subscribe::{min_qos, publish_to_client},
};

pub struct SubscribeExclusive<S> {
    metadata_cache: Arc<MetadataCacheManager>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    subscribe_manager: Arc<SubscribeManager>,
    message_storage: Arc<S>,
    ack_manager: Arc<AckManager>,
    // (client_id, Sender<bool>)
    push_thread: DashMap<String, Sender<bool>>,
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
        ack_manager: Arc<AckManager>,
    ) -> Self {
        return SubscribeExclusive {
            message_storage,
            metadata_cache,
            response_queue_sx4,
            response_queue_sx5,
            push_thread: DashMap::with_capacity(256),
            subscribe_manager,
            ack_manager,
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
        for (_, sub_list) in self.subscribe_manager.exclusive_subscribe.clone() {
            for subscribe in sub_list {
                let client_id = subscribe.client_id.clone();
                let thread_key = format!("{}_{}", client_id, subscribe.topic_id);

                if self.push_thread.contains_key(&thread_key) {
                    continue;
                }
                let (stop_sx, mut stop_rx) = broadcast::channel(2);
                let response_queue_sx4 = self.response_queue_sx4.clone();
                let response_queue_sx5 = self.response_queue_sx5.clone();
                let metadata_cache = self.metadata_cache.clone();
                let message_storage = self.message_storage.clone();
                let ack_manager = self.ack_manager.clone();

                // Subscribe to the data push thread
                self.push_thread.insert(thread_key, stop_sx);
                tokio::spawn(async move {
                    info(format!(
                        "Exclusive push thread for client_id [{}],topic_id [{}] was started successfully",
                        client_id, subscribe.topic_id
                    ));
                    let message_storage = MessageStorage::new(message_storage);
                    let group_id = format!("system_sub_{}_{}", client_id, subscribe.topic_id);
                    let record_num = 5;
                    let max_wait_ms = 100;

                    let mut sub_ids = Vec::new();
                    if let Some(id) = subscribe.subscription_identifier {
                        sub_ids.push(id);
                    }

                    loop {
                        match stop_rx.try_recv() {
                            Ok(flag) => {
                                if flag {
                                    info(format!(
                                        "Exclusive Push thread for client_id [{}],topic_id [{}] was stopped successfully",
                                        client_id.clone(),
                                    subscribe.topic_id
                                    ));

                                    break;
                                }
                            }
                            Err(_) => {}
                        }

                        let connect_id = if let Some(id) =
                            metadata_cache.get_connect_id(subscribe.client_id.clone())
                        {
                            id
                        } else {
                            continue;
                        };

                        match message_storage
                            .read_topic_message(
                                subscribe.topic_id.clone(),
                                group_id.clone(),
                                record_num,
                            )
                            .await
                        {
                            Ok(result) => {
                                if result.len() == 0 {
                                    sleep(Duration::from_millis(max_wait_ms)).await;
                                    continue;
                                }

                                for record in result.clone() {
                                    let msg = match Message::decode_record(record.clone()) {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            error(e.to_string());
                                            continue;
                                        }
                                    };

                                    if subscribe.nolocal && (subscribe.client_id == msg.client_id) {
                                        continue;
                                    }

                                    let qos = min_qos(msg.qos, subscribe.qos);

                                    let retain = if subscribe.preserve_retain {
                                        msg.retain
                                    } else {
                                        false
                                    };

                                    let pkid = metadata_cache.get_available_pkid(client_id.clone());
                                    let publish = Publish {
                                        dup: false,
                                        qos,
                                        pkid,
                                        retain,
                                        topic: Bytes::from(subscribe.topic_name.clone()),
                                        payload: Bytes::from(msg.payload),
                                    };

                                    let properties = PublishProperties {
                                        payload_format_indicator: None,
                                        message_expiry_interval: None,
                                        topic_alias: None,
                                        response_topic: None,
                                        correlation_data: None,
                                        user_properties: Vec::new(),
                                        subscription_identifiers: sub_ids.clone(),
                                        content_type: None,
                                    };

                                    let resp = ResponsePackage {
                                        connection_id: connect_id,
                                        packet: MQTTPacket::Publish(publish, Some(properties)),
                                    };

                                    match publish_to_client(
                                        subscribe.protocol.clone(),
                                        resp,
                                        response_queue_sx4.clone(),
                                        response_queue_sx5.clone(),
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            match qos {
                                                protocol::mqtt::QoS::AtMostOnce => {
                                                    match message_storage
                                                        .commit_group_offset(
                                                            subscribe.topic_id.clone(),
                                                            group_id.clone(),
                                                            record.offset,
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
                                                // protocol::mqtt::QoS::AtLeastOnce
                                                // protocol::mqtt::QoS::ExactlyOnce
                                                _ => {
                                                    metadata_cache
                                                        .save_pkid_info(client_id.clone(), pkid);
                                                    let (qos_sx, qos_rx) = mpsc::channel(1);
                                                    ack_manager.add(
                                                        client_id.clone(),
                                                        pkid,
                                                        AckPacketInfo {
                                                            sx: qos_sx,
                                                            create_time: now_second(),
                                                            client_id: client_id.clone(),
                                                            packet: resp,
                                                        },
                                                    );

                                                    if wait_qos_ack(qos_rx).await {
                                                        match message_storage
                                                            .commit_group_offset(
                                                                subscribe.topic_id.clone(),
                                                                group_id.clone(),
                                                                record.offset,
                                                            )
                                                            .await
                                                        {
                                                            Ok(_) => {}
                                                            Err(e) => {
                                                                error(e.to_string());
                                                                continue;
                                                            }
                                                        }
                                                    } else {
                                                        ack_manager.remove(client_id.clone(), pkid);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error(e.to_string());
                                            continue;
                                        }
                                    };
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
        }
    }
}

async fn wait_qos_ack(mut rx: mpsc::Receiver<bool>) -> bool {
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
