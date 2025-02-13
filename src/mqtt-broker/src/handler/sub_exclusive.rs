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

use super::cache::CacheManager;
use crate::subscribe::subscribe_manager::SubscribeManager;
use common_base::utils::topic_util::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub};

use metadata_struct::mqtt::cluster::AvailableFlag;
use protocol::mqtt::common::{Subscribe, Unsubscribe};
use std::sync::Arc;

pub fn check_exclusive_subscribe(
    metadata_cache: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    subscribe: &Subscribe,
) -> bool {
    if metadata_cache
        .get_cluster_info()
        .feature
        .exclusive_subscription_available
        == AvailableFlag::Disable
    {
        return false;
    }

    for filter in subscribe.filters.clone() {
        if !is_exclusive_sub(&filter.path) {
            continue;
        }

        let topic_name = decode_exclusive_sub_path_to_topic_name(&filter.path);
        if subscribe_manager.is_exclusive_subscribe(topic_name) {
            return false;
        }
    }

    true
}

pub fn add_exclusive_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
    path: &str,
    client_id: &str,
) {
    if is_exclusive_sub(path) {
        return;
    }

    let topic_name = decode_exclusive_sub_path_to_topic_name(path);
    subscribe_manager.add_exclusive_subscribe(topic_name, client_id);
}

pub fn remove_exclusive_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
    un_subscribe: Unsubscribe,
) {
    for path in un_subscribe.filters {
        if is_exclusive_sub(&path) {
            return;
        }

        let topic_name = decode_exclusive_sub_path_to_topic_name(&path);
        subscribe_manager.remove_exclusive_subscribe_by_topic(topic_name);
    }
}

pub fn remove_exclusive_subscribe_by_path(subscribe_manager: &Arc<SubscribeManager>, path: &str) {
    if is_exclusive_sub(path) {
        return;
    }

    let topic_name = decode_exclusive_sub_path_to_topic_name(path);
    subscribe_manager.remove_exclusive_subscribe_by_topic(topic_name);
}
