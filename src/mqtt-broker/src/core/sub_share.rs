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

use crate::core::cache::MQTTCacheManager;
use broker_core::share_group::ShareGroupStorage;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;

pub const SHARE_SUB_PREFIX: &str = "$share";

pub fn is_mqtt_share_subscribe(path: &str) -> bool {
    path.starts_with(SHARE_SUB_PREFIX)
}

pub fn decode_share_info(path: &str) -> (String, String) {
    let mut parts = path.splitn(3, '/');
    if parts.next() != Some(SHARE_SUB_PREFIX) {
        return (String::new(), String::new());
    }

    let Some(group_name) = parts.next() else {
        return (String::new(), String::new());
    };
    let Some(topic_filter) = parts.next() else {
        return (String::new(), String::new());
    };

    (group_name.to_string(), topic_filter.to_string())
}

/// Build the stable share-group key as `<group>/<topic-filter>`.
///
/// The separator is explicit because `decode_share_info` preserves whether the
/// MQTT topic filter itself starts with `/`.
pub fn full_group_name(group_name: &str, sub_name: &str) -> String {
    format!("{group_name}/{sub_name}")
}

pub async fn is_share_sub_leader(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    group_name: &str,
) -> Result<bool, CommonError> {
    let conf = broker_config();

    if let Some(group) = cache_manager.node_cache.get_share_group(tenant, group_name) {
        return Ok(group.leader_broker == conf.broker_id);
    }

    let storage = ShareGroupStorage::new(client_pool.clone());
    match storage.get(tenant, group_name).await? {
        Some(group) => {
            let is_leader = group.leader_broker == conf.broker_id;
            cache_manager.node_cache.add_share_group(group);
            Ok(is_leader)
        }
        None => Ok(false),
    }
}

pub async fn get_share_sub_leader(
    cache_manager: &Arc<MQTTCacheManager>,
    tenant: &str,
    group_name: &str,
) -> Option<u64> {
    if let Some(group) = cache_manager.node_cache.get_share_group(tenant, group_name) {
        return Some(group.leader_broker);
    }
    None
}

/// Resolve the leader broker id of a share group, falling back to the meta store
/// (and warming the cache) when the local cache hasn't received the group yet.
pub async fn resolve_share_sub_leader_id(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    group_name: &str,
) -> Result<Option<u64>, CommonError> {
    if let Some(group) = cache_manager.node_cache.get_share_group(tenant, group_name) {
        return Ok(Some(group.leader_broker));
    }

    let storage = ShareGroupStorage::new(client_pool.clone());
    match storage.get(tenant, group_name).await? {
        Some(group) => {
            let leader = group.leader_broker;
            cache_manager.node_cache.add_share_group(group);
            Ok(Some(leader))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_share_info() {
        assert_eq!(
            decode_share_info("$share/consumer1/sport/tennis/+"),
            ("consumer1".to_string(), "sport/tennis/+".to_string())
        );
        assert_eq!(
            decode_share_info("$share/group/a/b/c"),
            ("group".to_string(), "a/b/c".to_string())
        );
        assert_eq!(
            decode_share_info("$share/g/t"),
            ("g".to_string(), "t".to_string())
        );
        assert_eq!(
            decode_share_info("$share/g//t"),
            ("g".to_string(), "/t".to_string())
        );
        assert_eq!(decode_share_info(""), ("".to_string(), "".to_string()));
        assert_eq!(
            decode_share_info("$share"),
            ("".to_string(), "".to_string())
        );
        assert_eq!(
            decode_share_info("$share/group"),
            ("".to_string(), "".to_string())
        );
        assert_eq!(
            decode_share_info("share/g/t"),
            ("".to_string(), "".to_string())
        );
    }

    #[test]
    fn test_full_group_name() {
        assert_eq!(full_group_name("g", "a/b"), "g/a/b");
        assert_eq!(full_group_name("g", "/a/b"), "g//a/b");
    }

    #[test]
    fn test_is_mqtt_share_subscribe() {
        assert!(is_mqtt_share_subscribe("$share/g/t"));
        assert!(is_mqtt_share_subscribe("$share"));
        assert!(!is_mqtt_share_subscribe("share/g/t"));
        assert!(!is_mqtt_share_subscribe(""));
    }
}
