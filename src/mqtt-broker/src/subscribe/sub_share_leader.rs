use super::{
    sub_common::{min_qos, share_sub_rewrite_publish_flag},
    sub_exclusive::{publish_message_qos0, publish_message_qos1, publish_message_qos2},
    subscribe_cache::SubscribeCache,
};
use crate::{
    core::metadata_cache::MetadataCacheManager,
    metadata::{message::Message, subscriber::Subscriber},
    qos::ack_manager::{AckManager, AckPacketInfo},
    server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info},
    tools::now_second,
};
use futures::future::err;
use protocol::mqtt::{Publish, PublishProperties, QoS};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    sync::broadcast::{self},
    time::sleep,
};

const SHARED_SUBSCRIPTION_STRATEGY_ROUND_ROBIN: &str = "round_robin";
const SHARED_SUBSCRIPTION_STRATEGY_RANDOM: &str = "random";
const SHARED_SUBSCRIPTION_STRATEGY_STICKY: &str = "sticky";
const SHARED_SUBSCRIPTION_STRATEGY_HASH: &str = "hash";
const SHARED_SUBSCRIPTION_STRATEGY_LOCAL: &str = "local";

#[derive(Clone)]
pub struct SubscribeShareLeader<S> {
    pub subscribe_manager: Arc<SubscribeCache>,
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
        subscribe_manager: Arc<SubscribeCache>,
        message_storage: Arc<S>,
        response_queue_sx4: broadcast::Sender<ResponsePackage>,
        response_queue_sx5: broadcast::Sender<ResponsePackage>,
        metadata_cache: Arc<MetadataCacheManager>,
        ack_manager: Arc<AckManager>,
    ) -> Self {
        return SubscribeShareLeader {
            subscribe_manager,
            message_storage,
            response_queue_sx4,
            response_queue_sx5,
            metadata_cache,
            ack_manager,
        };
    }

    pub async fn start(&self) {
        loop {
            self.start_push_thread();
            self.try_thread_gc();
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn try_thread_gc(&self) {
        // Periodically verify that a push task is running, but the subscribe task has stopped
        // If so, stop the process and clean up the data
        for (share_leader_key, sx) in self.subscribe_manager.share_leader_push_thread.clone() {
            if !self
                .subscribe_manager
                .share_leader_subscribe
                .contains_key(&share_leader_key)
            {
                match sx.send(true) {
                    Ok(_) => {
                        self.subscribe_manager
                            .share_leader_push_thread
                            .remove(&share_leader_key);
                    }
                    Err(err) => {
                        error(format!(
                            "stop sub share thread error, error message:{}",
                            err.to_string()
                        ));
                    }
                }
            }
        }
    }

    pub fn start_push_thread(&self) {
        let conf = broker_mqtt_conf();
        // Periodically verify if any push tasks are not started. If so, the thread is started
        for (share_leader_key, sub_data) in self.subscribe_manager.share_leader_subscribe.clone() {
            // start push data thread
            let subscribe_manager = self.subscribe_manager.clone();
            if !self
                .subscribe_manager
                .share_leader_push_thread
                .contains_key(&share_leader_key)
            {
                // round_robin
                if conf.subscribe.shared_subscription_strategy
                    == SHARED_SUBSCRIPTION_STRATEGY_ROUND_ROBIN.to_string()
                {
                    self.push_by_round_robin(
                        share_leader_key.clone(),
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
                    self.push_by_random();
                }

                // sticky
                if conf.subscribe.shared_subscription_strategy
                    == SHARED_SUBSCRIPTION_STRATEGY_STICKY.to_string()
                {
                    self.push_by_sticky();
                }

                // hash
                if conf.subscribe.shared_subscription_strategy
                    == SHARED_SUBSCRIPTION_STRATEGY_HASH.to_string()
                {
                    self.push_by_hash();
                }

                // local
                if conf.subscribe.shared_subscription_strategy
                    == SHARED_SUBSCRIPTION_STRATEGY_LOCAL.to_string()
                {
                    self.push_by_local();
                }
            }
        }
    }

    fn push_by_round_robin(
        &self,
        share_leader_key: String,
        group_name: String,
        topic_id: String,
        topic_name: String,
        subscribe_manager: Arc<SubscribeCache>,
    ) {
        let (stop_sx, mut stop_rx) = broadcast::channel(1);
        self.subscribe_manager
            .share_leader_push_thread
            .insert(share_leader_key.clone(), stop_sx.clone());

        let response_queue_sx4 = self.response_queue_sx4.clone();
        let response_queue_sx5 = self.response_queue_sx5.clone();
        let metadata_cache = self.metadata_cache.clone();
        let message_storage = self.message_storage.clone();
        let ack_manager = self.ack_manager.clone();

        tokio::spawn(async move {
            info(format!(
                "Share leader push data thread for GroupName {},Topic [{}] was started successfully",
                group_name, topic_name
            ));

            let message_storage = MessageStorage::new(message_storage);
            let group_id = format!("system_sub_{}_{}", group_name, topic_id);

            let max_wait_ms = 500;
            let cursor_point = 0;
            let mut sub_list =
                build_share_leader_sub_list(subscribe_manager.clone(), share_leader_key.clone());
            let mut pre_update_sub_list_time = now_second();
            let mut record_num = calc_record_num(sub_list.len());
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
                if (now_second() - pre_update_sub_list_time) > 5 {
                    sub_list = build_share_leader_sub_list(
                        subscribe_manager.clone(),
                        share_leader_key.clone(),
                    );
                    pre_update_sub_list_time = now_second();
                    record_num = calc_record_num(sub_list.len());
                }

                match message_storage
                    .read_topic_message(topic_id.clone(), group_id.clone(), record_num as u128)
                    .await
                {
                    Ok(results) => {
                        if results.len() == 0 {
                            sleep(Duration::from_millis(max_wait_ms)).await;
                            continue;
                        }
                        for record in results {
                            let msg: Message = match Message::decode_record(record.clone()) {
                                Ok(msg) => msg,
                                Err(e) => {
                                    error(e.to_string());
                                    continue;
                                }
                            };

                            let current_point = if cursor_point < sub_list.len() {
                                cursor_point
                            } else {
                                0
                            };

                            let subscribe = sub_list.get(current_point).unwrap();

                            let mut sub_id = Vec::new();
                            if let Some(id) = subscribe.subscription_identifier {
                                sub_id.push(id);
                            }

                            let qos = min_qos(msg.qos, subscribe.qos);

                            let mut publish = Publish {
                                dup: false,
                                qos: qos.clone(),
                                pkid: 0,
                                retain: false,
                                topic: Bytes::from(topic_name.clone()),
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

                            match qos {
                                QoS::AtMostOnce => {
                                    publish_message_qos0(
                                        metadata_cache.clone(),
                                        subscribe.client_id.clone(),
                                        publish,
                                        properties,
                                        subscribe.protocol.clone(),
                                        response_queue_sx4.clone(),
                                        response_queue_sx5.clone(),
                                        stop_sx.clone(),
                                    )
                                    .await;
                                }

                                QoS::AtLeastOnce => {
                                    let pkid: u16 =
                                        metadata_cache.get_pkid(subscribe.client_id.clone()).await;
                                    publish.pkid = pkid;

                                    let (wait_puback_sx, _) = broadcast::channel(1);
                                    ack_manager.add(
                                        subscribe.client_id.clone(),
                                        pkid,
                                        AckPacketInfo {
                                            sx: wait_puback_sx.clone(),
                                            create_time: now_second(),
                                        },
                                    );

                                    match publish_message_qos1(
                                        metadata_cache.clone(),
                                        subscribe.client_id.clone(),
                                        publish.clone(),
                                        properties.clone(),
                                        pkid,
                                        subscribe.protocol.clone(),
                                        response_queue_sx4.clone(),
                                        response_queue_sx5.clone(),
                                        stop_sx.clone(),
                                        wait_puback_sx,
                                    )
                                    .await
                                    {
                                        Ok(()) => {
                                            metadata_cache.remove_pkid_info(
                                                subscribe.client_id.clone(),
                                                pkid,
                                            );
                                            ack_manager.remove(subscribe.client_id.clone(), pkid);
                                        }
                                        Err(e) => {
                                            error(e.to_string());
                                        }
                                    }
                                }
                                QoS::ExactlyOnce => {
                                    let pkid: u16 =
                                        metadata_cache.get_pkid(subscribe.client_id.clone()).await;
                                    publish.pkid = pkid;

                                    let (wait_ack_sx, _) = broadcast::channel(1);
                                    ack_manager.add(
                                        subscribe.client_id.clone(),
                                        pkid,
                                        AckPacketInfo {
                                            sx: wait_ack_sx.clone(),
                                            create_time: now_second(),
                                        },
                                    );
                                    match publish_message_qos2(
                                        metadata_cache.clone(),
                                        subscribe.client_id.clone(),
                                        publish,
                                        properties,
                                        pkid,
                                        subscribe.protocol.clone(),
                                        response_queue_sx4.clone(),
                                        response_queue_sx5.clone(),
                                        stop_sx.clone(),
                                        wait_ack_sx,
                                    )
                                    .await
                                    {
                                        Ok(()) => {
                                            metadata_cache.remove_pkid_info(
                                                subscribe.client_id.clone(),
                                                pkid,
                                            );
                                            ack_manager.remove(subscribe.client_id.clone(), pkid);
                                        }
                                        Err(e) => {
                                            error(e.to_string());
                                        }
                                    }
                                }
                            };

                            // commit offset
                            match message_storage
                                .commit_group_offset(
                                    topic_id.clone(),
                                    group_id.clone(),
                                    record.offset,
                                )
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    error(e.to_string());
                                }
                            }
                            continue;
                        }
                    }
                    Err(e) => {
                        error(format!(
                        "Failed to read message from storage, failure message: {},topic:{},group{}",
                        e.to_string(),
                        topic_id.clone(),
                        group_id.clone()
                    ));
                        sleep(Duration::from_millis(max_wait_ms)).await;
                        continue;
                    }
                }
            }

            subscribe_manager
                .share_leader_push_thread
                .remove(&share_leader_key);
        });
    }

    fn push_by_random(&self) {}

    fn push_by_hash(&self) {}

    fn push_by_sticky(&self) {}

    fn push_by_local(&self) {}
}

fn build_share_leader_sub_list(
    subscribe_manager: Arc<SubscribeCache>,
    key: String,
) -> Vec<Subscriber> {
    let sub_list = if let Some(sub) = subscribe_manager.share_leader_subscribe.get(&key) {
        sub.sub_list.clone()
    } else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for (_, sub) in sub_list {
        result.push(sub);
    }
    return result;
}

fn calc_record_num(sub_len: usize) -> usize {
    return sub_len * 5;
}

#[cfg(test)]
mod tests {}
