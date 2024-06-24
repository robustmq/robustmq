use super::cache_manager::{CacheManager, QosAckPacketInfo};
use crate::{
    server::tcp::packet::ResponsePackage,
    storage::topic::TopicStorage,
    subscribe::{
        sub_common::{get_sub_topic_id_list, min_qos, publish_message_qos0},
        sub_exclusive::{exclusive_publish_message_qos1, exclusive_publish_message_qos2},
    },
};
use bytes::Bytes;
use clients::poll::ClientPool;
use common_base::{errors::RobustMQError, log::error, tools::now_second};
use metadata_struct::mqtt::message::MQTTMessage;
use protocol::mqtt::common::{
    Publish, PublishProperties, QoS, RetainForwardRule, Subscribe, SubscribeProperties,
};
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};

pub async fn save_topic_retain_message(
    topic_name: String,
    client_id: String,
    publish: Publish,
    cache_manager: Arc<CacheManager>,
    publish_properties: Option<PublishProperties>,
    client_poll: Arc<ClientPool>,
) -> Result<(), RobustMQError> {
    if publish.retain {
        let topic_storage = TopicStorage::new(client_poll.clone());
        if publish.payload.is_empty() {
            match topic_storage
                .delete_retain_message(topic_name.clone())
                .await
            {
                Ok(_) => {
                    cache_manager.update_topic_retain_message(&topic_name, Some(Vec::new()));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            let retain_message = MQTTMessage::build_message(
                client_id.clone(),
                publish.clone(),
                publish_properties.clone(),
            );
            let message_expire =
                now_second() + message_expiry_interval(cache_manager.clone(), publish_properties);
            match topic_storage
                .set_retain_message(topic_name.clone(), retain_message.clone(), message_expire)
                .await
            {
                Ok(_) => {
                    cache_manager
                        .update_topic_retain_message(&topic_name, Some(retain_message.encode()));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    return Ok(());
}

// Reservation messages are processed when a subscription is created
pub async fn send_retain_message(
    client_id: String,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    client_poll: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    response_queue_sx: Sender<ResponsePackage>,
    new_sub: bool,
    stop_sx: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut sub_id = Vec::new();
        if let Some(properties) = subscribe_properties {
            if let Some(id) = properties.subscription_identifier {
                sub_id.push(id);
            }
        }

        for filter in subscribe.filters {
            if filter.retain_forward_rule == RetainForwardRule::Never {
                continue;
            }

            if filter.retain_forward_rule == RetainForwardRule::OnNewSubscribe && !new_sub {
                continue;
            }

            let topic_id_list = get_sub_topic_id_list(cache_manager.clone(), filter.path).await;
            let topic_storage = TopicStorage::new(client_poll.clone());
            let cluster = cache_manager.get_cluster_info();
            for topic_id in topic_id_list {
                match topic_storage.get_retain_message(topic_id.clone()).await {
                    Ok(Some(msg)) => {
                        if filter.nolocal && client_id == msg.client_id {
                            continue;
                        }

                        let retain = if filter.preserve_retain {
                            msg.retain
                        } else {
                            false
                        };

                        if let Some(topic_name) = cache_manager.topic_name_by_id(topic_id) {
                            let qos = min_qos(cluster.max_qos, filter.qos);
                            let pkid = 1;
                            let mut publish = Publish {
                                dup: false,
                                qos,
                                pkid,
                                retain,
                                topic: Bytes::from(topic_name),
                                payload: msg.payload,
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

                            tokio::spawn(async {});
                            match qos {
                                QoS::AtMostOnce => {
                                    publish_message_qos0(
                                        cache_manager.clone(),
                                        client_id.clone(),
                                        publish,
                                        response_queue_sx.clone(),
                                        stop_sx.clone(),
                                    )
                                    .await;
                                }

                                QoS::AtLeastOnce => {
                                    let pkid: u16 = cache_manager.get_pkid(client_id.clone()).await;
                                    publish.pkid = pkid;

                                    let (wait_puback_sx, _) = broadcast::channel(1);
                                    cache_manager.add_ack_packet(
                                        client_id.clone(),
                                        pkid,
                                        QosAckPacketInfo {
                                            sx: wait_puback_sx.clone(),
                                            create_time: now_second(),
                                        },
                                    );

                                    match exclusive_publish_message_qos1(
                                        cache_manager.clone(),
                                        client_id.clone(),
                                        publish,
                                        properties,
                                        pkid,
                                        response_queue_sx.clone(),
                                        stop_sx.clone(),
                                        wait_puback_sx,
                                    )
                                    .await
                                    {
                                        Ok(()) => {
                                            cache_manager.remove_pkid_info(client_id.clone(), pkid);
                                            cache_manager
                                                .remove_ack_packet(client_id.clone(), pkid);
                                        }
                                        Err(e) => {
                                            error(e.to_string());
                                        }
                                    }
                                }

                                QoS::ExactlyOnce => {
                                    let pkid: u16 = cache_manager.get_pkid(client_id.clone()).await;
                                    publish.pkid = pkid;

                                    let (wait_ack_sx, _) = broadcast::channel(1);
                                    cache_manager.add_ack_packet(
                                        client_id.clone(),
                                        pkid,
                                        QosAckPacketInfo {
                                            sx: wait_ack_sx.clone(),
                                            create_time: now_second(),
                                        },
                                    );
                                    match exclusive_publish_message_qos2(
                                        cache_manager.clone(),
                                        client_id.clone(),
                                        publish,
                                        properties,
                                        pkid,
                                        response_queue_sx.clone(),
                                        stop_sx.clone(),
                                        wait_ack_sx,
                                    )
                                    .await
                                    {
                                        Ok(()) => {
                                            cache_manager.remove_pkid_info(client_id.clone(), pkid);
                                            cache_manager
                                                .remove_ack_packet(client_id.clone(), pkid);
                                        }
                                        Err(e) => {
                                            error(e.to_string());
                                        }
                                    }
                                }
                            };
                        }
                    }
                    Ok(None) => {
                        continue;
                    }
                    Err(e) => error(e.to_string()),
                }
            }
        }
    });
}

pub fn message_expiry_interval(
    cache_manager: Arc<CacheManager>,
    publish_properties: Option<PublishProperties>,
) -> u64 {
    let cluster = cache_manager.get_cluster_info();
    if let Some(properties) = publish_properties {
        if let Some(expire) = properties.message_expiry_interval {
            return std::cmp::min(cluster.max_message_expiry_interval, expire as u64);
        }
    }
    return cluster.default_message_expiry_interval;
}

#[cfg(test)]
mod tests {
    use crate::core::cache_manager::CacheManager;
    use crate::core::message_retain::send_retain_message;
    use crate::storage::topic::TopicStorage;
    use bytes::Bytes;
    use clients::poll::ClientPool;
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::message::MQTTMessage;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::{Filter, MQTTPacket, QoS, Subscribe, SubscribeProperties};
    use std::sync::Arc;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn send_retain_message_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(100));
        let metadata_cache = Arc::new(CacheManager::new(
            client_poll,
            "test-cluster".to_string(),
            4,
        ));
        let (response_queue_sx, mut response_queue_rx) = broadcast::channel(1000);
        let connect_id = 1;
        let mut filters = Vec::new();
        let flt = Filter {
            path: "/test/topic".to_string(),
            qos: QoS::AtLeastOnce,
            nolocal: true,
            preserve_retain: true,
            retain_forward_rule: protocol::mqtt::common::RetainForwardRule::OnEverySubscribe,
        };
        filters.push(flt);
        let subscribe = Subscribe {
            packet_identifier: 1,
            filters,
        };
        let subscribe_properties = Some(SubscribeProperties::default());

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let topic_storage = TopicStorage::new(client_poll.clone());
        let new_sub = true;

        let topic_name = "/test/topic".to_string();
        let payload = "testtesttest".to_string();
        let topic = MQTTTopic::new(unique_id(), topic_name.clone());
        let client_id = "xxxxx".to_string();

        metadata_cache.add_topic(&topic_name, &topic);

        let mut retain_message = MQTTMessage::default();
        retain_message.dup = false;
        retain_message.qos = QoS::AtLeastOnce;
        retain_message.pkid = 1;
        retain_message.retain = true;
        retain_message.topic = Bytes::from(topic_name.clone());
        retain_message.payload = Bytes::from(payload);

        match topic_storage
            .set_retain_message(topic.topic_id, retain_message.clone(), 10000)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }

        let (stop_send, _) = broadcast::channel(2);
        send_retain_message(
            client_id,
            subscribe,
            subscribe_properties,
            client_poll,
            metadata_cache.clone(),
            response_queue_sx.clone(),
            new_sub,
            stop_send,
        )
        .await;

        loop {
            match response_queue_rx.recv().await {
                Ok(packet) => {
                    if let MQTTPacket::Publish(publish, _) = packet.packet {
                        assert_eq!(publish.topic, retain_message.topic);
                        assert_eq!(publish.payload, retain_message.payload);
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
