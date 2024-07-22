use super::cache_manager::{CacheManager, QosAckPacketInfo};
use crate::{
    server::connection_manager::ConnectionManager,
    storage::topic::TopicStorage,
    subscribe::{
        sub_common::{get_sub_topic_id_list, min_qos, publish_message_qos0},
        sub_exclusive::{exclusive_publish_message_qos1, exclusive_publish_message_qos2},
        subscriber::Subscriber,
    },
};
use bytes::Bytes;
use clients::poll::ClientPool;
use common_base::{errors::RobustMQError, log::error, tools::now_second};
use metadata_struct::mqtt::message::MQTTMessage;
use protocol::mqtt::common::{Publish, PublishProperties, QoS, RetainForwardRule};
use std::sync::Arc;
use tokio::sync::broadcast::{self};

pub async fn save_topic_retain_message(
    cache_manager: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    topic_name: &String,
    client_id: &String,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Result<(), RobustMQError> {
    if publish.retain {
        let topic_storage = TopicStorage::new(client_poll.clone());
        if publish.payload.is_empty() {
            match topic_storage.delete_retain_message(topic_name).await {
                Ok(_) => {
                    cache_manager.update_topic_retain_message(&topic_name, Some(Vec::new()));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            let retain_message = MQTTMessage::build_message(client_id, publish, publish_properties);
            let message_expire = message_expiry_interval(cache_manager, publish_properties);
            match topic_storage
                .set_retain_message(topic_name, &retain_message, message_expire)
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
pub async fn try_send_retain_message(
    client_id: String,
    subscriber: Subscriber,
    client_poll: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    stop_sx: broadcast::Sender<bool>,
) {
    let sub_id = if let Some(id) = subscriber.subscription_identifier {
        vec![id]
    } else {
        Vec::new()
    };

    if subscriber.retain_forward_rule == RetainForwardRule::Never {
        return;
    }

    let is_new_sub = cache_manager.is_new_sub(&client_id, &subscriber.sub_path);

    if subscriber.retain_forward_rule == RetainForwardRule::OnNewSubscribe && is_new_sub {
        return;
    }

    tokio::spawn(async move {
        let topic_id_list = get_sub_topic_id_list(cache_manager.clone(), subscriber.sub_path).await;
        let topic_storage = TopicStorage::new(client_poll.clone());
        let cluster = cache_manager.get_cluster_info();
        for topic_id in topic_id_list {
            if let Some(topic_name) = cache_manager.topic_name_by_id(topic_id) {
                match topic_storage.get_retain_message(topic_name.clone()).await {
                    Ok(Some(msg)) => {
                        if subscriber.nolocal && client_id == msg.client_id {
                            continue;
                        }

                        let retain = if subscriber.preserve_retain {
                            msg.retain
                        } else {
                            false
                        };

                        let qos = min_qos(cluster.max_qos, subscriber.qos);
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
                            payload_format_indicator: msg.format_indicator,
                            message_expiry_interval: msg.expiry_interval,
                            topic_alias: None,
                            response_topic: msg.response_topic,
                            correlation_data: msg.correlation_data,
                            user_properties: msg.user_properties,
                            subscription_identifiers: sub_id.clone(),
                            content_type: msg.content_type,
                        };

                        tokio::spawn(async {});
                        match qos {
                            QoS::AtMostOnce => {
                                publish_message_qos0(
                                    &cache_manager,
                                    &client_id,
                                    &publish,
                                    &Some(properties),
                                    &connection_manager,
                                    &stop_sx,
                                )
                                .await;
                            }

                            QoS::AtLeastOnce => {
                                let pkid: u16 = cache_manager.get_pkid(&client_id).await;
                                publish.pkid = pkid;

                                let (wait_puback_sx, _) = broadcast::channel(1);
                                cache_manager.add_ack_packet(
                                    &client_id,
                                    pkid,
                                    QosAckPacketInfo {
                                        sx: wait_puback_sx.clone(),
                                        create_time: now_second(),
                                    },
                                );

                                match exclusive_publish_message_qos1(
                                    &cache_manager,
                                    &client_id,
                                    publish,
                                    &properties,
                                    pkid,
                                    &connection_manager,
                                    &stop_sx,
                                    &wait_puback_sx,
                                )
                                .await
                                {
                                    Ok(()) => {
                                        cache_manager.remove_pkid_info(&client_id, pkid);
                                        cache_manager.remove_ack_packet(&client_id, pkid);
                                    }
                                    Err(e) => {
                                        error(e.to_string());
                                    }
                                }
                            }

                            QoS::ExactlyOnce => {
                                let pkid: u16 = cache_manager.get_pkid(&client_id).await;
                                publish.pkid = pkid;

                                let (wait_ack_sx, _) = broadcast::channel(1);
                                cache_manager.add_ack_packet(
                                    &client_id,
                                    pkid,
                                    QosAckPacketInfo {
                                        sx: wait_ack_sx.clone(),
                                        create_time: now_second(),
                                    },
                                );
                                match exclusive_publish_message_qos2(
                                    &cache_manager,
                                    &client_id,
                                    &publish,
                                    &properties,
                                    pkid,
                                    &connection_manager,
                                    &stop_sx,
                                    &wait_ack_sx,
                                )
                                .await
                                {
                                    Ok(()) => {
                                        cache_manager.remove_pkid_info(&client_id, pkid);
                                        cache_manager.remove_ack_packet(&client_id, pkid);
                                    }
                                    Err(e) => {
                                        error(e.to_string());
                                    }
                                }
                            }
                        };
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
    cache_manager: &Arc<CacheManager>,
    publish_properties: &Option<PublishProperties>,
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
    use crate::handler::retain::try_send_retain_message;
    use crate::storage::topic::TopicStorage;
    use crate::subscribe::subscriber::Subscriber;
    use crate::{
        handler::cache_manager::CacheManager, server::connection_manager::ConnectionManager,
    };
    use bytes::Bytes;
    use clients::poll::ClientPool;
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::message::MQTTMessage;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::mqtt::common::{Filter, QoS, Subscribe, SubscribeProperties};
    use std::sync::Arc;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn send_retain_message_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(100));
        let metadata_cache = Arc::new(CacheManager::new(client_poll, "test-cluster".to_string()));

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
            .set_retain_message(&topic.topic_id, &retain_message, 10000)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }
        let connection_manager = Arc::new(ConnectionManager::new(metadata_cache.clone()));

        let (stop_send, _) = broadcast::channel(1);
        let subscriber = Subscriber::default();
        try_send_retain_message(
            client_id,
            subscriber,
            client_poll,
            metadata_cache.clone(),
            connection_manager,
            stop_send,
        )
        .await;
    }
}
