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

use super::{cache::CacheManager, error::MqttBrokerError};
use crate::subscribe::{
    sub_common::{delete_exclusive_topic, set_nx_exclusive_topic},
    subscribe_manager::SubscribeManager,
};
use common_base::{error::common::CommonError, utils::topic_util};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::cluster::AvailableFlag;
use protocol::mqtt::common::{Subscribe, SubscribeReasonCode, Unsubscribe};
use std::sync::Arc;

pub fn is_exclusive_subscribe() {}

pub async fn save_exclusive_subscribe(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    subscribe: &Subscribe,
) -> Result<Option<SubscribeReasonCode>, MqttBrokerError> {
    for filter in subscribe.filters.clone() {
        if !topic_util::is_exclusive_sub(&filter.path) {
            continue;
        }

        if metadata_cache
            .get_cluster_info()
            .feature
            .exclusive_subscription_available
            == AvailableFlag::Disable
        {
            return Ok(Some(SubscribeReasonCode::ExclusiveSubscriptionDisabled));
        }

        let topic_name = topic_util::decode_exclusive_sub_path_to_topic_name(&filter.path);

        if !set_nx_exclusive_topic(client_pool.clone(), topic_name.to_owned())
            .await?
            .success
        {
            return Ok(Some(SubscribeReasonCode::TopicSubscribed));
        }
    }
    Ok(None)
}

pub async fn remove_exclusive_subscribe(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    un_subscribe: Unsubscribe,
) -> Result<(), CommonError> {
    for filter in un_subscribe.filters.clone() {
        remove_exclusive_subscribe_by_sub_path(client_pool, metadata_cache, filter.clone()).await?;
    }
    Ok(())
}

pub async fn remove_exclusive_subscribe_by_client_id(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
) -> Result<(), CommonError> {
    for (_, subscriber) in subscribe_manager.exclusive_subscribe.clone() {
        if subscriber.client_id == *client_id {
            remove_exclusive_subscribe_by_sub_path(
                client_pool,
                metadata_cache,
                subscriber.sub_path,
            )
            .await?;
        }
    }
    Ok(())
}

async fn remove_exclusive_subscribe_by_sub_path(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    un_sub_path: String,
) -> Result<(), CommonError> {
    if !topic_util::is_exclusive_sub(&un_sub_path) {
        return Ok(());
    }

    if metadata_cache
        .get_cluster_info()
        .feature
        .exclusive_subscription_available
        == AvailableFlag::Disable
    {
        return Ok(());
    }

    let topic_name = topic_util::decode_exclusive_sub_path_to_topic_name(&un_sub_path);
    delete_exclusive_topic(client_pool.clone(), topic_name.to_owned()).await?;
    Ok(())
}
