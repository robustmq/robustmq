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
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use protocol::mqtt::common::{Filter, Login, MqttProtocol, Subscribe};
use std::sync::Arc;

pub async fn try_auto_subscribe(
    client_id: String,
    login: &Option<Login>,
    protocol: &MqttProtocol,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
) -> ResultMqttBrokerError {
    let auto_subscribe_rules: Vec<MqttAutoSubscribeRule> = cache_manager
        .auto_subscribe_rule
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    let username: String = if let Some(login_info) = login {
        login_info.username.clone()
    } else {
        "".to_string()
    };

    let mut filters: Vec<Filter> = Vec::new();
    for auto_subscribe_rule in auto_subscribe_rules {
        let mut path: String = auto_subscribe_rule.topic.clone();
        path = path.replace("${clientid}", &client_id);
        if !username.is_empty() {
            path = path.replace("${username}", &username);
        }

        filters.push(Filter {
            path,
            qos: auto_subscribe_rule.qos,
            no_local: auto_subscribe_rule.no_local,
            preserve_retain: auto_subscribe_rule.retain_as_published,
            retain_handling: auto_subscribe_rule.retained_handling,
        });
    }

    if !filters.is_empty() {
        let subscribe: Subscribe = Subscribe {
            packet_identifier: 0,
            filters: filters.clone(),
        };

        match save_subscribe(SaveSubscribeContext {
            client_id: client_id.clone(),
            protocol: protocol.clone(),
            client_pool: client_pool.clone(),
            cache_manager: cache_manager.clone(),
            subscribe_manager: subscribe_manager.clone(),
            subscribe: subscribe.clone(),
            subscribe_properties: None,
        })
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        };
    }
    Ok(())
}
