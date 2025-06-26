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

use super::error::MqttBrokerError;
use crate::subscribe::{
    common::{
        decode_queue_info, decode_share_info, is_queue_sub, is_share_sub,
        SHARE_QUEUE_DEFAULT_GROUP_NAME,
    },
    manager::SubscribeManager,
};

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::{placement::mqtt::call::placement_delete_subscribe, pool::ClientPool};
use protocol::{
    mqtt::common::Unsubscribe, placement_center::placement_center_mqtt::DeleteSubscribeRequest,
};
use std::sync::Arc;

pub async fn remove_subscribe(
    client_id: &str,
    un_subscribe: &Unsubscribe,
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<SubscribeManager>,
) -> Result<(), MqttBrokerError> {
    let conf = broker_mqtt_conf();

    for path in un_subscribe.filters.clone() {
        let request = DeleteSubscribeRequest {
            cluster_name: conf.cluster_name.to_owned(),
            client_id: client_id.to_owned(),
            path: path.clone(),
        };

        placement_delete_subscribe(client_pool, &conf.placement_center, request).await?;

        subscribe_manager.remove_subscribe(client_id, &path);
    }

    unsubscribe_by_path(subscribe_manager, client_id, &un_subscribe.filters)?;

    Ok(())
}

fn unsubscribe_by_path(
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
    filter_path: &[String],
) -> Result<(), MqttBrokerError> {
    for path in filter_path {
        if is_share_sub(path) && is_queue_sub(path) {
            let (group_name, sub_name) = if is_queue_sub(path) {
                (
                    SHARE_QUEUE_DEFAULT_GROUP_NAME.to_string(),
                    decode_queue_info(path),
                )
            } else {
                decode_share_info(path)
            };

            // share leader
            for (_, data) in subscribe_manager.share_leader_push.clone() {
                if !(data.group_name == group_name && data.sub_name == sub_name) {
                    continue;
                }
                if let Some((_, share_sub)) = data.sub_list.remove(client_id) {
                    subscribe_manager.remove_topic_subscribe_by_path(
                        &share_sub.topic_name,
                        client_id,
                        &share_sub.sub_path,
                    );
                }
            }

            // share follower
            for (key, data) in subscribe_manager.share_follower_resub.clone() {
                if data.client_id == *client_id && data.filter.path == *path {
                    subscribe_manager.share_follower_resub.remove(&key);
                    if let Some(sx) = subscribe_manager.share_follower_resub_thread.get(&key) {
                        sx.sender.send(true)?;
                    }
                }
            }
        } else {
            for (key, subscriber) in subscribe_manager.exclusive_push.clone() {
                if subscriber.client_id == *client_id && subscriber.sub_path == *path {
                    if let Some(sx) = subscribe_manager.exclusive_push_thread.get(&key) {
                        sx.sender.send(true)?;
                        subscribe_manager.exclusive_push.remove(&key);
                    }
                    subscribe_manager.remove_topic_subscribe_by_path(
                        &subscriber.topic_name,
                        client_id,
                        &subscriber.sub_path,
                    );
                }
            }
        }
    }
    Ok(())
}
