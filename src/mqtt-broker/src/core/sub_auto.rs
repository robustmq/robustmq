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
use crate::core::tool::ResultMqttBrokerError;
use crate::subscribe::manager::SubscribeManager;
use grpc_clients::pool::ClientPool;
use protocol::mqtt::common::{Filter, Login, MqttProtocol, Subscribe};
use std::sync::Arc;
use tracing::{debug, error, info};

fn replace_topic_placeholders(
    pattern: &str,
    client_id: &str,
    username: &str,
    remote_addr: &str,
) -> String {
    pattern
        .replace("${clientid}", client_id)
        .replace("${username}", username)
        .replace("${host}", remote_addr)
}

pub async fn try_auto_subscribe(
    client_id: String,
    login: &Option<Login>,
    remote_addr: String,
    protocol: &MqttProtocol,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
) -> ResultMqttBrokerError {
    if cache_manager.auto_subscribe_rule.is_empty() {
        return Ok(());
    }

    let username = login
        .as_ref()
        .map(|login_info| login_info.username.as_str())
        .unwrap_or("");

    let mut filters: Vec<Filter> = Vec::new();

    for rule_entry in cache_manager.auto_subscribe_rule.iter() {
        let rule = rule_entry.value();
        let topic = replace_topic_placeholders(&rule.topic, &client_id, username, &remote_addr);

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
        info!(
            "Applying {} auto-subscription rule(s) for client: {}",
            filters.len(),
            client_id
        );

        let subscribe = Subscribe {
            packet_identifier: 0,
            filters,
        };

        if let Err(e) = save_subscribe(SaveSubscribeContext {
            client_id: client_id.clone(),
            protocol: protocol.clone(),
            client_pool: client_pool.clone(),
            cache_manager: cache_manager.clone(),
            subscribe_manager: subscribe_manager.clone(),
            subscribe,
            subscribe_properties: None,
        })
        .await
        {
            error!(
                "Failed to apply auto-subscription for client {}: {:?}",
                client_id, e
            );
            return Err(e);
        }
    }

    Ok(())
}
