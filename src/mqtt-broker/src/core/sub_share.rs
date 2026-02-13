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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::{meta::mqtt::call::placement_get_share_sub_leader, pool::ClientPool};
use protocol::meta::meta_service_mqtt::{GetShareSubLeaderRequest, SubLeaderInfo};
use std::{collections::HashMap, sync::Arc, vec};

pub const SHARE_SUB_PREFIX: &str = "$share";

pub fn is_mqtt_share_subscribe(path: &str) -> bool {
    path.starts_with(SHARE_SUB_PREFIX)
}

pub fn decode_share_info(path: &str) -> (String, String) {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() < 3 || parts[0] != "$share" {
        return (String::new(), String::new());
    }

    let group_name = parts[1].to_string();
    let topic_path = format!("/{}", parts[2..].join("/"));
    (group_name, topic_path)
}

pub fn full_group_name(group_name: &str, sub_name: &str) -> String {
    format!("{group_name}{sub_name}")
}

pub async fn is_share_sub_leader(
    client_pool: &Arc<ClientPool>,
    group_name: &str,
) -> Result<bool, CommonError> {
    let reply = fetch_share_sub_leader(client_pool, vec![group_name.to_string()]).await?;
    let conf = broker_config();
    if let Some(raw) = reply.get(group_name) {
        return Ok(raw.broker_id == conf.broker_id);
    }
    Err(CommonError::CommonError(format!(
        "share subscription leader not found for group: {group_name}"
    )))
}

pub async fn get_share_sub_leader(
    client_pool: &Arc<ClientPool>,
    group_name: &str,
) -> Result<u64, CommonError> {
    let reply = fetch_share_sub_leader(client_pool, vec![group_name.to_string()]).await?;
    if let Some(raw) = reply.get(group_name) {
        return Ok(raw.broker_id);
    }
    Err(CommonError::CommonError(format!(
        "share subscription leader not found for group: {group_name}"
    )))
}

pub async fn fetch_share_sub_leader(
    client_pool: &Arc<ClientPool>,
    group_name: Vec<String>,
) -> Result<HashMap<String, SubLeaderInfo>, CommonError> {
    let conf = broker_config();
    let req = GetShareSubLeaderRequest {
        group_list: group_name.to_owned(),
    };
    let reply =
        placement_get_share_sub_leader(client_pool, &conf.get_meta_service_addr(), req).await?;
    let mut results = HashMap::new();
    for raw in reply.leader.iter() {
        results.insert(raw.group_name.clone(), raw.clone());
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_share_info() {
        assert_eq!(
            decode_share_info("$share/consumer1/sport/tennis/+"),
            ("consumer1".to_string(), "/sport/tennis/+".to_string())
        );
        assert_eq!(
            decode_share_info("$share/group/a/b/c"),
            ("group".to_string(), "/a/b/c".to_string())
        );
        assert_eq!(
            decode_share_info("$share/g/t"),
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
    fn test_is_mqtt_share_subscribe() {
        assert!(is_mqtt_share_subscribe("$share/g/t"));
        assert!(is_mqtt_share_subscribe("$share"));
        assert!(!is_mqtt_share_subscribe("share/g/t"));
        assert!(!is_mqtt_share_subscribe(""));
    }
}
