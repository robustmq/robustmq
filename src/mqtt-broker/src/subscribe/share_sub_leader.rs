use super::{
    sub_manager::SubscribeManager,
    subscribe::{min_qos, retry_publish, share_sub_rewrite_publish_flag},
};
use crate::{
    core::metadata_cache::MetadataCacheManager, metadata::message::Message,
    qos::ack_manager::AckManager, server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info},
};
use dashmap::DashMap;
use protocol::mqtt::{MQTTPacket, Publish, PublishProperties};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, Sender},
    },
    time::sleep,
};

const SHARED_SUBSCRIPTION_STRATEGY_ROUND_ROBIN: &str = "round_robin";
const SHARED_SUBSCRIPTION_STRATEGY_RANDOM: &str = "random";
const SHARED_SUBSCRIPTION_STRATEGY_STICKY: &str = "sticky";
const SHARED_SUBSCRIPTION_STRATEGY_HASH: &str = "hash";
const SHARED_SUBSCRIPTION_STRATEGY_LOCAL: &str = "local";

#[derive(Clone)]
pub struct SubscribeShareLeader<S> {
    // (group_name_topic_name, Sender<bool>)
    pub leader_pull_data_thread: DashMap<String, Sender<bool>>,

    // (group_name_topic_name, Sender<bool>)
    pub leader_push_data_thread: DashMap<String, Sender<bool>>,

    pub subscribe_manager: Arc<SubscribeManager>,
    message_storage: Arc<S>,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
    metadata_cache: Arc<MetadataCacheManager>,
    ack_manager: Arc<AckManager>,
}

impl<S> SubscribeShareLeader<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        message_storage: Arc<S>,
        response_queue_sx4: broadcast::Sender<ResponsePackage>,
        response_queue_sx5: broadcast::Sender<ResponsePackage>,
        metadata_cache: Arc<MetadataCacheManager>,
        ack_manager: Arc<AckManager>,
    ) -> Self {
        return SubscribeShareLeader {
            leader_pull_data_thread: DashMap::with_capacity(128),
            leader_push_data_thread: DashMap::with_capacity(128),
            subscribe_manager,
            message_storage,
            response_queue_sx4,
            response_queue_sx5,
            metadata_cache,
            ack_manager,
        };
    }

    // leader_push_sub_check
    pub async fn start_leader_push_sub_check_thread(&self) {
        let conf = broker_mqtt_conf();
        loop {
            // Periodically verify if any push tasks are not started
            // If so, the thread is started
            for (_, sub_data) in self.subscribe_manager.share_leader_subscribe.clone() {
                let key = format!("{}_{}", sub_data.group_name, sub_data.topic_id);
                if sub_data.sub_list.len() == 0 {
                    // stop pull data thread
                    if let Some(sx) = self.leader_pull_data_thread.get(&key) {
                        match sx.send(true).await {
                            Ok(_) => {
                                self.leader_pull_data_thread.remove(&sub_data.topic_id);
                            }
                            Err(e) => error(e.to_string()),
                        }
                    }

                    // stop push data thread
                    if let Some(sx) = self.leader_push_data_thread.get(&key) {
                        match sx.send(true).await {
                            Ok(_) => {
                                self.leader_push_data_thread.remove(&key);
                            }
                            Err(e) => error(e.to_string()),
                        }
                    }

                    continue;
                }

                // start push data thread
                let subscribe_manager = self.subscribe_manager.clone();
                if !self.leader_push_data_thread.contains_key(&key) {
                    // round_robin
                    if conf.subscribe.shared_subscription_strategy
                        == SHARED_SUBSCRIPTION_STRATEGY_ROUND_ROBIN.to_string()
                    {
                        self.start_push_by_round_robin(
                            sub_data.group_name.clone(),
                            sub_data.topic_id.clone(),
                            sub_data.topic_name.clone(),
                            subscribe_manager,
                        );
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

            // Periodically verify that a push task is running, but the subscribe task has stopped
            // If so, stop the process and clean up the data
            for (key, sx) in self.leader_push_data_thread.clone() {
                if !self
                    .subscribe_manager
                    .share_leader_subscribe
                    .contains_key(&key)
                {
                    match sx.send(true).await {
                        Ok(()) => {
                            self.leader_push_data_thread.remove(&key);
                        }
                        Err(_) => {}
                    }
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn start_push_by_round_robin(
        &self,
        group_name: String,
        topic_id: String,
        topic_name: String,
        subscribe_manager: Arc<SubscribeManager>,
    ) {
        let (stop_sx, mut stop_rx) = mpsc::channel(1);
        self.leader_push_data_thread
            .insert(topic_id.clone(), stop_sx);
        let response_queue_sx4 = self.response_queue_sx4.clone();
        let response_queue_sx5 = self.response_queue_sx5.clone();
        let metadata_cache = self.metadata_cache.clone();
        let message_storage = self.message_storage.clone();
        let ack_manager = self.ack_manager.clone();

        tokio::spawn(async move {
            info(format!(
                "Share sub push data thread for GroupName {},Topic [{}] was started successfully",
                group_name, topic_name
            ));

            let message_storage = MessageStorage::new(message_storage);
            let group_id = format!("system_sub_{}_{}", group_name, topic_id);

            let max_wait_ms = 500;
            let cursor_point = 0;

            loop {
                match stop_rx.try_recv() {
                    Ok(flag) => {
                        if flag {
                            info(format!(
                                "Share sub push data thread for GroupName {},Topic [{}] was stopped successfully",
                                group_name, topic_name
                            ));
                            break;
                        }
                    }
                    Err(_) => {}
                }

                let sub_list = if let Some(sub) =
                    subscribe_manager.share_leader_subscribe.get(&topic_id)
                {
                    sub.sub_list
                } else {
                    info(format!(
                            "Share sub push data thread for GroupName {},Topic [{}] , subscription length is 0 and the thread is exited.",
                            group_name, topic_name
                        ));
                    break;
                };

                let record_num = sub_list.len() * 2;
                match message_storage
                    .read_topic_message(topic_id.clone(), group_id.clone(), record_num as u128)
                    .await
                {
                    Ok(results) => {
                        if results.len() == 0 {
                            sleep(Duration::from_millis(max_wait_ms)).await;
                            return;
                        }
                        for record in results {
                            let msg: Message = match Message::decode_record(record) {
                                Ok(msg) => msg,
                                Err(e) => {
                                    error(e.to_string());
                                    return;
                                }
                            };

                            let current_point = if cursor_point < sub_list.len() {
                                cursor_point
                            } else {
                                0
                            };

                            let subscribe = sub_list.get(current_point).unwrap();
                            
                            let pkid: u16 =
                                metadata_cache.get_available_pkid(subscribe.client_id.clone());

                            let connect_id = if let Some(connect_id) =
                                metadata_cache.get_connect_id(subscribe.client_id)
                            {
                                connect_id
                            } else {
                                continue;
                            };

                            let mut sub_id = Vec::new();
                            if let Some(id) = subscribe.subscription_identifier {
                                sub_id.push(id);
                            }

                            let qos = min_qos(msg.qos, subscribe.qos);

                            let publish = Publish {
                                dup: false,
                                qos: qos.clone(),
                                pkid,
                                retain: false,
                                topic: Bytes::from(topic_name),
                                payload: msg.payload,
                            };

                            let mut user_properteis = Vec::new();
                            if subscribe.is_contain_rewrite_flag {
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
                            let resp: ResponsePackage = ResponsePackage {
                                connection_id: connect_id,
                                packet: MQTTPacket::Publish(publish, Some(properties)),
                            };

                            match retry_publish(
                                subscribe.client_id.clone(),
                                pkid,
                                metadata_cache.clone(),
                                ack_manager.clone(),
                                qos,
                                subscribe.clone(),
                                resp,
                                response_queue_sx4.clone(),
                                response_queue_sx5.clone(),
                            )
                            .await
                            {
                                Ok(()) => {
                                    cursor_point = cursor_point + 1;
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

    fn start_push_by_random(&self) {}

    fn start_push_by_hash(&self) {}

    fn start_push_by_sticky(&self) {}

    fn start_push_by_local(&self) {}
}

#[cfg(test)]
mod tests {}
