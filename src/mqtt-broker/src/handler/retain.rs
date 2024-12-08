// Copyright 2023 RobustMQ Team
//
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

use std::sync::Arc;

use bytes::Bytes;
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use log::info;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{
    MqttProtocol, Publish, PublishProperties, QoS, RetainForwardRule, Subscribe,
    SubscribeProperties,
};
use tokio::sync::broadcast::{self};

use super::cache::{CacheManager, QosAckPacketInfo};
use super::constant::{SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE};
use super::error::MqttBrokerError;
use super::message::build_message_expire;
use crate::observability::metrics::packets::{
    record_retain_recv_metrics, record_retain_sent_metrics,
};
use crate::server::connection_manager::ConnectionManager;
use crate::storage::topic::TopicStorage;
use crate::subscribe::sub_common::{get_sub_topic_id_list, min_qos, publish_message_qos0};
use crate::subscribe::sub_exclusive::{
    exclusive_publish_message_qos1, exclusive_publish_message_qos2,
};
use crate::subscribe::subscriber::Subscriber;
use crate::subscribe::SubPublishParam;

pub async fn save_retain_message(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: String,
    client_id: &str,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Result<(), MqttBrokerError> {
    if !publish.retain {
        return Ok(());
    }

    let topic_storage = TopicStorage::new(client_pool.clone());

    if publish.payload.is_empty() {
        topic_storage
            .delete_retain_message(topic_name.clone())
            .await?;
        cache_manager.update_topic_retain_message(&topic_name, Some(Vec::new()));
    } else {
        record_retain_recv_metrics(publish.qos);
        let message_expire = build_message_expire(cache_manager, publish_properties);
        let retain_message =
            MqttMessage::build_message(client_id, publish, publish_properties, message_expire);
        topic_storage
            .set_retain_message(topic_name.clone(), &retain_message, message_expire)
            .await?;

        cache_manager.update_topic_retain_message(&topic_name, Some(retain_message.encode()));
    }

    Ok(())
}

pub async fn try_send_retain_message(
    protocol: MqttProtocol,
    client_id: String,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
) {
    tokio::spawn(async move {
        let (stop_sx, _) = broadcast::channel(1);
        if let Err(e) = send_retain_message(
            &protocol,
            &client_id,
            &subscribe,
            &subscribe_properties,
            &client_pool,
            &cache_manager,
            &connection_manager,
            &stop_sx,
        )
        .await
        {
            info!("Sending retain message failed with error message :{}", e);
        }
    });
}

#[allow(clippy::too_many_arguments)]
async fn send_retain_message(
    protocol: &MqttProtocol,
    client_id: &String,
    subscribe: &Subscribe,
    subscribe_properties: &Option<SubscribeProperties>,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
) -> Result<(), MqttBrokerError> {
    let mut sub_ids = Vec::new();
    if let Some(properties) = subscribe_properties {
        if let Some(id) = properties.subscription_identifier {
            sub_ids.push(id);
        }
    }

    for filter in subscribe.filters.iter() {
        if filter.retain_forward_rule == RetainForwardRule::Never {
            return Ok(());
        }

        let is_new_sub = cache_manager.is_new_sub(client_id, &filter.path);
        if filter.retain_forward_rule == RetainForwardRule::OnNewSubscribe && !is_new_sub {
            return Ok(());
        }

        let topic_id_list = get_sub_topic_id_list(cache_manager, &filter.path).await;
        let topic_storage = TopicStorage::new(client_pool.clone());
        let cluster = cache_manager.get_cluster_info();
        for topic_id in topic_id_list.iter() {
            let topic_name = if let Some(topic_name) = cache_manager.topic_name_by_id(topic_id) {
                topic_name
            } else {
                continue;
            };

            let msg = if let Some(message) = topic_storage.get_retain_message(&topic_name).await? {
                message
            } else {
                continue;
            };

            if filter.nolocal && *client_id == msg.client_id {
                continue;
            }

            let retain = if filter.preserve_retain {
                msg.retain
            } else {
                false
            };

            let qos = min_qos(cluster.protocol.max_qos, filter.qos);

            let mut user_properties = msg.user_properties;
            user_properties.push((
                SUB_RETAIN_MESSAGE_PUSH_FLAG.to_string(),
                SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE.to_string(),
            ));

            let properties = PublishProperties {
                payload_format_indicator: msg.format_indicator,
                message_expiry_interval: Some(msg.expiry_interval as u32),
                topic_alias: None,
                response_topic: msg.response_topic,
                correlation_data: msg.correlation_data,
                user_properties,
                subscription_identifiers: sub_ids.clone(),
                content_type: msg.content_type,
            };

            let pkid = if qos != QoS::AtMostOnce {
                cache_manager.get_pkid(client_id).await
            } else {
                0
            };

            let publish = Publish {
                dup: false,
                qos,
                pkid,
                retain,
                topic: Bytes::from(topic_name.clone()),
                payload: msg.payload,
            };

            let subscriber = Subscriber {
                protocol: protocol.to_owned(),
                client_id: client_id.clone(),
                ..Default::default()
            };
            let sub_pub_param = SubPublishParam::new(
                subscriber,
                publish,
                Some(properties),
                msg.create_time as u128,
                "".to_string(),
                pkid,
            );

            match qos {
                QoS::AtMostOnce => {
                    publish_message_qos0(
                        cache_manager,
                        connection_manager,
                        &sub_pub_param,
                        stop_sx,
                    )
                    .await;
                }

                QoS::AtLeastOnce => {
                    let (wait_puback_sx, _) = broadcast::channel(1);
                    cache_manager.add_ack_packet(
                        client_id,
                        pkid,
                        QosAckPacketInfo {
                            sx: wait_puback_sx.clone(),
                            create_time: now_second(),
                        },
                    );

                    exclusive_publish_message_qos1(
                        cache_manager,
                        connection_manager,
                        &sub_pub_param,
                        stop_sx,
                        &wait_puback_sx,
                    )
                    .await?;

                    cache_manager.remove_pkid_info(client_id, pkid);
                    cache_manager.remove_ack_packet(client_id, pkid);
                }

                QoS::ExactlyOnce => {
                    let (wait_ack_sx, _) = broadcast::channel(1);
                    cache_manager.add_ack_packet(
                        client_id,
                        pkid,
                        QosAckPacketInfo {
                            sx: wait_ack_sx.clone(),
                            create_time: now_second(),
                        },
                    );

                    exclusive_publish_message_qos2(
                        cache_manager,
                        connection_manager,
                        &sub_pub_param,
                        stop_sx,
                        &wait_ack_sx,
                    )
                    .await?;

                    cache_manager.remove_pkid_info(client_id, pkid);
                    cache_manager.remove_ack_packet(client_id, pkid);
                }
            };

            record_retain_sent_metrics(qos);
        }
    }
    Ok(())
}
