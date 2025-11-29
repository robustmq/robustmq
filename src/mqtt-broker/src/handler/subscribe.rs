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

use super::{cache::MQTTCacheManager, error::MqttBrokerError};
use crate::common::types::ResultMqttBrokerError;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::parse::{parse_subscribe_by_new_subscribe, ParseSubscribeContext};
use common_base::tools::now_second;
use common_config::broker::broker_config;
use grpc_clients::meta::mqtt::call::placement_delete_subscribe;
use grpc_clients::{meta::mqtt::call::placement_set_subscribe, pool::ClientPool};
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use protocol::meta::meta_service_mqtt::DeleteSubscribeRequest;
use protocol::mqtt::common::Unsubscribe;
use protocol::{
    meta::meta_service_mqtt::SetSubscribeRequest,
    mqtt::common::{MqttProtocol, Subscribe, SubscribeProperties},
};
use std::sync::Arc;
use tracing::error;

#[derive(Clone)]
pub struct SaveSubscribeContext {
    pub client_id: String,
    pub protocol: MqttProtocol,
    pub client_pool: Arc<ClientPool>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
}

pub async fn save_subscribe(context: SaveSubscribeContext) -> ResultMqttBrokerError {
    let conf = broker_config();
    let filters = &context.subscribe.filters;
    for filter in filters {
        let subscribe_data = MqttSubscribe {
            client_id: context.client_id.to_owned(),
            path: filter.path.clone(),
            cluster_name: conf.cluster_name.to_owned(),
            broker_id: conf.broker_id,
            filter: filter.clone(),
            pkid: context.subscribe.packet_identifier,
            subscribe_properties: context.subscribe_properties.to_owned(),
            protocol: context.protocol.to_owned(),
            create_time: now_second(),
        };

        let request = SetSubscribeRequest {
            cluster_name: conf.cluster_name.to_owned(),
            client_id: context.client_id.to_owned(),
            path: filter.path.clone(),
            subscribe: subscribe_data.encode()?,
        };

        if let Err(e) =
            placement_set_subscribe(&context.client_pool, &conf.get_meta_service_addr(), request)
                .await
        {
            error!(
                "Failed to set subscribe to meta service, error message: {}",
                e
            );
            return Err(MqttBrokerError::CommonError(e.to_string()));
        }
    }
    Ok(())
}

pub async fn remove_subscribe(
    client_id: &str,
    un_subscribe: &Unsubscribe,
    client_pool: &Arc<ClientPool>,
) -> ResultMqttBrokerError {
    let conf = broker_config();
    for path in un_subscribe.filters.clone() {
        let request = DeleteSubscribeRequest {
            cluster_name: conf.cluster_name.to_owned(),
            client_id: client_id.to_owned(),
            path: path.clone(),
        };

        placement_delete_subscribe(client_pool, &conf.get_meta_service_addr(), request).await?;
    }

    Ok(())
}

pub async fn actually_execute_remove_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
    subscribe: &MqttSubscribe,
) -> ResultMqttBrokerError {
    subscribe_manager.remove_by_sub(&subscribe.client_id, &subscribe.path);
    Ok(())
}

pub async fn actually_execute_add_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    subscribe: &MqttSubscribe,
) -> ResultMqttBrokerError {
    subscribe_manager.add_subscribe(subscribe);
    let rewrite_sub_path = cache_manager.get_new_rewrite_name(&subscribe.filter.path);
    if let Some(row) = cache_manager.topic_info.iter().next() {
        let topic = row.value();
        return parse_subscribe_by_new_subscribe(ParseSubscribeContext {
            client_pool: client_pool.clone(),
            subscribe_manager: subscribe_manager.clone(),
            client_id: subscribe.client_id.clone(),
            topic: topic.clone(),
            protocol: subscribe.protocol.clone(),
            pkid: subscribe.pkid,
            filter: subscribe.filter.clone(),
            subscribe_properties: subscribe.subscribe_properties.clone(),
            rewrite_sub_path,
        })
        .await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
