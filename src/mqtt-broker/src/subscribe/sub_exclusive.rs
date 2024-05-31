use crate::{
    core::metadata_cache::MetadataCacheManager,
    metadata::message::Message,
    qos::ack_manager::{AckManager, AckPackageData, AckPackageType, AckPacketInfo},
    server::{tcp::packet::ResponsePackage, MQTTProtocol},
    storage::message::MessageStorage,
};
use bytes::Bytes;
use common_base::{
    errors::RobustMQError,
    log::{error, info},
    tools::now_second,
};
use protocol::mqtt::{MQTTPacket, PubRel, Publish, PublishProperties, QoS};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    sync::broadcast::{self, Sender},
    time::sleep,
};

use super::{
    sub_common::{min_qos, publish_to_response_queue, wait_packet_ack},
    subscribe_cache::SubscribeCache,
};

pub struct SubscribeExclusive<S> {
    metadata_cache: Arc<MetadataCacheManager>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    subscribe_manager: Arc<SubscribeCache>,
    message_storage: Arc<S>,
    ack_manager: Arc<AckManager>,
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
        subscribe_manager: Arc<SubscribeCache>,
        ack_manager: Arc<AckManager>,
    ) -> Self {
        return SubscribeExclusive {
            message_storage,
            metadata_cache,
            response_queue_sx4,
            response_queue_sx5,
            subscribe_manager,
            ack_manager,
        };
    }

    pub async fn start(&self) {
        loop {
            self.start_push_thread().await;
            self.try_thread_gc().await;
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn try_thread_gc(&self) {
        // Periodically verify that a push task is running, but the subscribe task has stopped
        // If so, stop the process and clean up the data
        for (exclusive_key, sx) in self.subscribe_manager.exclusive_push_thread.clone() {
            if !self
                .subscribe_manager
                .exclusive_subscribe
                .contains_key(&exclusive_key)
            {
                match sx.send(true) {
                    Ok(_) => {
                        self.subscribe_manager
                            .exclusive_push_thread
                            .remove(&exclusive_key);
                    }
                    Err(_) => {}
                }
            }
        }
    }

    // Handles exclusive subscription push tasks
    // Exclusively subscribed messages are pushed directly to the consuming client
    async fn start_push_thread(&self) {
        for (exclusive_key, subscribe) in self.subscribe_manager.exclusive_subscribe.clone() {
            let client_id = subscribe.client_id.clone();

            if self
                .subscribe_manager
                .exclusive_push_thread
                .contains_key(&exclusive_key)
            {
                continue;
            }

            let (stop_sx, mut stop_rx) = broadcast::channel(2);
            let response_queue_sx4 = self.response_queue_sx4.clone();
            let response_queue_sx5 = self.response_queue_sx5.clone();
            let metadata_cache = self.metadata_cache.clone();
            let message_storage = self.message_storage.clone();
            let ack_manager = self.ack_manager.clone();
            let subscribe_manager = self.subscribe_manager.clone();

            // Subscribe to the data push thread
            self.subscribe_manager
                .exclusive_push_thread
                .insert(exclusive_key.clone(), stop_sx.clone());

            tokio::spawn(async move {
                info(format!(
                        "Exclusive push thread for client_id [{}],topic_id [{}] was started successfully",
                        client_id, subscribe.topic_id
                    ));
                let message_storage = MessageStorage::new(message_storage);
                let group_id = format!("system_sub_{}_{}", client_id, subscribe.topic_id);
                let record_num = 5;
                let max_wait_ms = 100;

                let cluster_qos = metadata_cache.get_cluster_info().max_qos();
                let qos = min_qos(cluster_qos, subscribe.qos);

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
                                        error(format!("Storage layer message Decord failed with error message :{}",e.to_string()));
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
                                            }
                                        }
                                        continue;
                                    }
                                };

                                if subscribe.nolocal && (subscribe.client_id == msg.client_id) {
                                    continue;
                                }

                                let retain = if subscribe.preserve_retain {
                                    msg.retain
                                } else {
                                    false
                                };

                                let mut publish = Publish {
                                    dup: false,
                                    qos,
                                    pkid: 1,
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

                                match qos {
                                    QoS::AtMostOnce => {
                                        publish_message_qos0(
                                            metadata_cache.clone(),
                                            client_id.clone(),
                                            publish,
                                            subscribe.protocol.clone(),
                                            response_queue_sx4.clone(),
                                            response_queue_sx5.clone(),
                                            stop_sx.clone(),
                                        )
                                        .await;
                                    }

                                    QoS::AtLeastOnce => {
                                        let pkid: u16 =
                                            metadata_cache.get_pkid(client_id.clone()).await;
                                        publish.pkid = pkid;

                                        let (wait_puback_sx, _) = broadcast::channel(1);
                                        ack_manager.add(
                                            client_id.clone(),
                                            pkid,
                                            AckPacketInfo {
                                                sx: wait_puback_sx.clone(),
                                                create_time: now_second(),
                                            },
                                        );

                                        match publish_message_qos1(
                                            metadata_cache.clone(),
                                            client_id.clone(),
                                            publish,
                                            properties,
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
                                                metadata_cache
                                                    .remove_pkid_info(client_id.clone(), pkid);
                                                ack_manager.remove(client_id.clone(), pkid);
                                            }
                                            Err(e) => {
                                                error(e.to_string());
                                            }
                                        }
                                    }
                                    QoS::ExactlyOnce => {
                                        let pkid: u16 =
                                            metadata_cache.get_pkid(client_id.clone()).await;
                                        publish.pkid = pkid;

                                        let (wait_ack_sx, _) = broadcast::channel(1);
                                        ack_manager.add(
                                            client_id.clone(),
                                            pkid,
                                            AckPacketInfo {
                                                sx: wait_ack_sx.clone(),
                                                create_time: now_second(),
                                            },
                                        );
                                        match publish_message_qos2(
                                            metadata_cache.clone(),
                                            client_id.clone(),
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
                                                metadata_cache
                                                    .remove_pkid_info(client_id.clone(), pkid);
                                                ack_manager.remove(client_id.clone(), pkid);
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
                                        subscribe.topic_id.clone(),
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
                                    subscribe.topic_id.clone(),
                                    group_id.clone()
                                ));
                            sleep(Duration::from_millis(max_wait_ms)).await;
                        }
                    }
                }

                subscribe_manager
                    .exclusive_push_thread
                    .remove(&exclusive_key);
            });
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
            Err(_) => {},
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
                "Failed to write QOS0 Publish message to response queue, failure message: {}",
                e.to_string()
            ));
        }
    }
}

// When the subscribed QOS is 1, we need to keep retrying to send the message to the client.
// To avoid messages that are not successfully pushed to the client. When the client Session expires,
// the push thread will exit automatically and will not attempt to push again.
pub async fn publish_message_qos1(
    metadata_cache: Arc<MetadataCacheManager>,
    client_id: String,
    mut publish: Publish,
    publish_properties: PublishProperties,
    pkid: u16,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    wait_puback_sx: broadcast::Sender<AckPackageData>,
) -> Result<(), RobustMQError> {
    let mut retry_times = 0;
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

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
            packet: MQTTPacket::Publish(publish.clone(), Some(publish_properties.clone())),
        };

        match publish_to_response_queue(
            protocol.clone(),
            resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                if let Some(data) = wait_packet_ack(wait_puback_sx.clone()).await {
                    if data.ack_type == AckPackageType::PubAck && data.pkid == pkid {
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                error(format!(
                    "Failed to write QOS1 Publish message to response queue, failure message: {}",
                    e.to_string()
                ));
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// send publish message
// wait pubrec message
// send pubrel message
// wait pubcomp message
pub async fn publish_message_qos2(
    metadata_cache: Arc<MetadataCacheManager>,
    client_id: String,
    mut publish: Publish,
    publish_properties: PublishProperties,
    pkid: u16,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    wait_ack_sx: broadcast::Sender<AckPackageData>,
) -> Result<(), RobustMQError> {
    // 1. send publish message
    let mut retry_times = 0;
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

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
            packet: MQTTPacket::Publish(publish.clone(), Some(publish_properties.clone())),
        };

        match publish_to_response_queue(
            protocol.clone(),
            resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                break;
            }
            Err(e) => {
                error(format!(
                    "Failed to write QOS1 Publish message to response queue, failure message: {}",
                    e.to_string()
                ));
                sleep(Duration::from_millis(5)).await;
            }
        }
    }

    // 2. wait pub rec
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        if let Some(data) = wait_packet_ack(wait_ack_sx.clone()).await {
            if data.ack_type == AckPackageType::PubRec && data.pkid == pkid {
                break;
            }
        }
    }

    // 3. send pub rel
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

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

        match publish_to_response_queue(
            protocol.clone(),
            pubrel_resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
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

    // 4. wait pub comp
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    break;
                }
            }
            Err(_) => {}
        }
        if let Some(data) = wait_packet_ack(wait_ack_sx.clone()).await {
            if data.ack_type == AckPackageType::PubComp && data.pkid == pkid {
                break;
            }
        }
    }
    return Ok(());
}
