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

use super::cache::MQTTCacheManager;
use super::constant::{
    MAX_RETAIN_MESSAGE_SEND_CONCURRENCY, SUB_RETAIN_MESSAGE_PUSH_FLAG,
    SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
};
use super::message::build_message_expire;
use crate::core::error::MqttBrokerError;
use crate::core::sub_option::is_send_retain_msg_by_retain_handling;
use crate::core::subscribe::is_new_sub;
use crate::core::tool::ResultMqttBrokerError;
use crate::storage::retain::RetainStorage;
use crate::subscribe::common::SubPublishParam;
use crate::subscribe::common::{client_unavailable_error, get_sub_topic_name_list};
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::push::send_publish_packet_to_client;
use bytes::Bytes;
use common_base::tools::now_second;
use common_metrics::mqtt::packets::{record_retain_recv_metrics, record_retain_sent_metrics};
use common_metrics::mqtt::statistics::{record_mqtt_retained_dec, record_mqtt_retained_inc};
use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{MqttPacket, Publish, PublishProperties, QoS, Subscribe};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::{broadcast, Semaphore};
use tracing::{debug, warn};

pub async fn save_retain_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    tenant: &str,
    topic_name: &str,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> ResultMqttBrokerError {
    if !publish.retain {
        return Ok(());
    }

    let topic_storage = RetainStorage::new(storage_driver_manager.clone());
    let had_retain = topic_storage
        .get_retain_message(tenant, topic_name)
        .await?
        .is_some();

    if had_retain && publish.payload.is_empty() {
        topic_storage
            .delete_retain_message(tenant, topic_name)
            .await?;
        record_mqtt_retained_dec();
        return Ok(());
    }

    if !publish.payload.is_empty() {
        record_retain_recv_metrics(publish.qos);
        if !had_retain {
            record_mqtt_retained_inc();
        }

        let expired_at = build_message_expire(cache_manager, publish_properties).await;
        let retain_message = MQTTRetainMessage {
            tenant: tenant.to_string(),
            topic_name: topic_name.to_string(),
            payload: publish.payload.clone(),
            expired_at,
            create_time: now_second(),
        };

        topic_storage
            .set_retain_message(tenant, topic_name, &retain_message)
            .await?;
    }

    Ok(())
}

pub struct SendRetainContext<'a> {
    pub storage_driver_manager: &'a Arc<StorageDriverManager>,
    pub cache_manager: &'a Arc<MQTTCacheManager>,
    pub connection_manager: &'a Arc<ConnectionManager>,
    pub subscribe_manager: &'a Arc<SubscribeManager>,
    pub tenant: &'a str,
    pub client_id: &'a str,
    pub subscribe: &'a Subscribe,
    pub stop_sx: &'a broadcast::Sender<bool>,
}

pub async fn try_send_retain_message(ctx: SendRetainContext<'_>) -> Result<(), MqttBrokerError> {
    let is_new_subs = is_new_sub(
        ctx.tenant,
        ctx.client_id,
        ctx.subscribe,
        ctx.subscribe_manager,
    );
    let semaphore = Arc::new(Semaphore::new(MAX_RETAIN_MESSAGE_SEND_CONCURRENCY));
    let mut handles = Vec::new();

    for filter in ctx.subscribe.filters.iter() {
        if !is_send_retain_msg_by_retain_handling(
            &filter.path,
            &filter.retain_handling,
            &is_new_subs,
        ) {
            debug!(
                "retain messages: Determine whether to send retained messages based on the \
                 retain handling strategy. Client ID: {}",
                ctx.client_id
            );
            continue;
        }

        let topic_name_list = get_sub_topic_name_list(ctx.cache_manager, &filter.path).await;

        for topic_name in topic_name_list {
            let storage = RetainStorage::new(ctx.storage_driver_manager.clone());
            let retain_message = match storage.get_retain_message(ctx.tenant, &topic_name).await? {
                Some(msg) => msg,
                None => continue,
            };

            if retain_message.expired_at > 0 && now_second() >= retain_message.expired_at {
                continue;
            }

            let semaphore_clone = semaphore.clone();
            let cache_manager = ctx.cache_manager.clone();
            let connection_manager = ctx.connection_manager.clone();
            let stop_sx = ctx.stop_sx.clone();
            let client_id = ctx.client_id.to_string();

            let handle = tokio::spawn(async move {
                let _permit = match semaphore_clone.acquire_owned().await {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to acquire semaphore for retain send: {}", e);
                        return;
                    }
                };

                let qos = QoS::AtLeastOnce;
                let p_kid = cache_manager
                    .pkid_manager
                    .generate_publish_to_client_pkid(&client_id, &qos)
                    .await;

                let publish = Publish {
                    dup: false,
                    qos,
                    p_kid,
                    retain: false,
                    topic: Bytes::copy_from_slice(retain_message.topic_name.as_bytes()),
                    payload: retain_message.payload.clone(),
                };

                let publish_properties = PublishProperties {
                    user_properties: vec![(
                        SUB_RETAIN_MESSAGE_PUSH_FLAG.to_string(),
                        SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE.to_string(),
                    )],
                    ..Default::default()
                };

                let packet = MqttPacket::Publish(publish, Some(publish_properties));
                let sub_pub_param = SubPublishParam {
                    packet,
                    create_time: now_second(),
                    client_id: client_id.clone(),
                    p_kid,
                    qos,
                };

                if let Err(e) = send_publish_packet_to_client(
                    &connection_manager,
                    &cache_manager,
                    &sub_pub_param,
                    &stop_sx,
                )
                .await
                {
                    if !client_unavailable_error(&e) {
                        warn!(
                            "Sending retain message failed: client_id={}, topic={}, error={}",
                            client_id, retain_message.topic_name, e
                        );
                    }
                } else {
                    record_retain_sent_metrics(qos);
                }
            });

            handles.push(handle);
        }
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
