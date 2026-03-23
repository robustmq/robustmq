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

use super::{
    cache::MQTTCacheManager,
    subscribe::{save_subscribe, SaveSubscribeContext},
};
use crate::core::tenant::try_decode_username;
use crate::core::tool::ResultMqttBrokerError;
use crate::subscribe::manager::SubscribeManager;
use grpc_clients::pool::ClientPool;
use protocol::mqtt::common::{Filter, Login, MqttProtocol, Subscribe};
use std::sync::Arc;
use tracing::debug;

fn replace_topic_placeholders(
    pattern: &str,
    client_id: &str,
    username: &str,
    remote_addr: &str,
) -> String {
    // Normalize IPv6 loopback to IPv4 loopback for consistency
    let host = if remote_addr == "::1" {
        "127.0.0.1"
    } else {
        remote_addr
    };
    pattern
        .replace("${clientid}", client_id)
        .replace("${username}", username)
        .replace("${host}", host)
}

#[allow(clippy::too_many_arguments)]
pub async fn try_auto_subscribe(
    client_id: String,
    tenant: &str,
    login: &Option<Login>,
    remote_addr: String,
    protocol: &MqttProtocol,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
) -> ResultMqttBrokerError {
    let tenant_rules = match cache_manager.auto_subscribe_rule.get(tenant) {
        Some(m) if !m.is_empty() => m,
        _ => return Ok(()),
    };

    let raw_username = login
        .as_ref()
        .map(|login_info| login_info.username.clone())
        .unwrap_or_default();
    let username = try_decode_username(&raw_username);

    let mut filters: Vec<Filter> = Vec::new();

    for rule_entry in tenant_rules.iter() {
        let rule = rule_entry.value();
        let topic = replace_topic_placeholders(&rule.topic, &client_id, &username, &remote_addr);

        debug!(
            "Auto-subscribe: client_id={}, original_pattern={}, resolved_topic={}, qos={:?}",
            client_id, rule.topic, topic, rule.qos
        );

        filters.push(Filter {
            path: topic,
            qos: rule.qos,
            no_local: rule.no_local,
            preserve_retain: rule.retain_as_published,
            retain_handling: rule.retained_handling.clone(),
        });
    }

    if !filters.is_empty() {
        debug!(
            "Applying {} auto-subscription rule(s) for client: {}",
            filters.len(),
            client_id
        );

        let subscribe = Subscribe {
            packet_identifier: 0,
            filters,
        };

        save_subscribe(SaveSubscribeContext {
            tenant: tenant.to_string(),
            client_id: client_id.clone(),
            protocol: protocol.clone(),
            client_pool: client_pool.clone(),
            cache_manager: cache_manager.clone(),
            subscribe_manager: subscribe_manager.clone(),
            subscribe,
            subscribe_properties: None,
        })
        .await?;
    }

    Ok(())
}
