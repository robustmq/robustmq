// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


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
    if !publish.retain {
        return Ok(());
    }

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
                    Err(e) => error(format!(
                        "send retain message error, error message:{}",
                        e.to_string()
                    )),
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
    return cluster.max_message_expiry_interval;
}

#[cfg(test)]
mod tests {
    use crate::handler::cache_manager::CacheManager;
    use clients::poll::ClientPool;
    use common_base::{
        config::broker_mqtt::init_broker_mqtt_conf_by_path, log::init_broker_mqtt_log,
    };
    use metadata_struct::mqtt::cluster::MQTTCluster;
    use protocol::mqtt::common::PublishProperties;
    use std::sync::Arc;

    use super::message_expiry_interval;

    #[tokio::test]
    async fn save_topic_retain_message_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        init_broker_mqtt_conf_by_path(&path);
        init_broker_mqtt_log();
    }

    #[test]
    fn message_expiry_interval_test() {
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let mut cluster = MQTTCluster::default();
        cluster.max_message_expiry_interval = 10;
        cache_manager.set_cluster_info(cluster);

        let publish_properties = None;
        let res = message_expiry_interval(&cache_manager, &publish_properties);
        assert_eq!(res, 10);

        let mut publish_properties = PublishProperties::default();
        publish_properties.message_expiry_interval = Some(3);
        let res = message_expiry_interval(&cache_manager, &Some(publish_properties));
        assert_eq!(res, 3);
    }
}
