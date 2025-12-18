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

use crate::controller::call_broker::call::{add_call_message, start_call_thread};
use crate::core::error::MetaServiceError;
use broker_core::cache::BrokerCacheManager;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_base::utils::serialize;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::resource_config::ResourceConfig;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker::broker_common::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType};
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tracing::error;
use tracing::warn;

#[derive(Clone)]
pub struct BrokerCallMessage {
    pub action_type: BrokerUpdateCacheActionType,
    pub resource_type: BrokerUpdateCacheResourceType,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct BrokerCallNodeSender {
    pub sender: Sender<BrokerCallMessage>,
    pub node: BrokerNode,
}

pub struct BrokerCallManager {
    pub node_sender: DashMap<u64, BrokerCallNodeSender>,
    pub node_stop_sender: DashMap<u64, Sender<bool>>,
    pub broker_cache: Arc<BrokerCacheManager>,
}

impl BrokerCallManager {
    pub fn new(broker_cache: Arc<BrokerCacheManager>) -> Self {
        let node_sender = DashMap::with_capacity(2);
        let node_stop_sender = DashMap::with_capacity(2);
        BrokerCallManager {
            node_sender,
            node_stop_sender,
            broker_cache,
        }
    }

    pub fn get_node_sender(&self, node_id: u64) -> Option<BrokerCallNodeSender> {
        if let Some(sender) = self.node_sender.get(&node_id) {
            return Some(sender.clone());
        }
        None
    }

    pub fn add_node_sender(&self, node_id: u64, sender: BrokerCallNodeSender) {
        self.node_sender.insert(node_id, sender);
    }

    pub fn remove_node(&self, node_id: u64) {
        self.node_sender.remove(&node_id);
        if let Some((_, send)) = self.node_stop_sender.remove(&node_id) {
            if let Err(e) = send.send(true) {
                warn!("{}", e);
            }
        }
    }

    pub fn add_node_stop_sender(&self, node_id: u64, sender: Sender<bool>) {
        self.node_stop_sender.insert(node_id, sender);
    }
}

pub async fn broker_call_thread_manager(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    stop: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        // start thread
        for (key, node_sender) in call_manager.node_sender.clone() {
            if !call_manager.node_stop_sender.contains_key(&key) {
                let (stop_send, _) = broadcast::channel(2);
                start_call_thread(
                    node_sender.node.clone(),
                    call_manager.clone(),
                    client_pool.clone(),
                    stop_send.clone(),
                )
                .await;
                call_manager.node_stop_sender.insert(key, stop_send);
            }
        }

        // gc thread
        for (key, sx) in call_manager.node_stop_sender.clone() {
            if !call_manager.node_sender.contains_key(&key) {
                if let Err(e) = sx.send(true) {
                    error!("node stop sender:{}", e);
                }
            }
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, 1000, &stop).await;
}

pub async fn update_cache_by_add_session(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Session,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_session(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::Session,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_schema(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&schema)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Schema,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_schema(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&schema)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::Schema,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_schema_bind(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&bind_data)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::SchemaResource,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_schema_bind(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&bind_data)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::SchemaResource,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_connector(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MQTTConnector,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Connector,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_connector(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MQTTConnector,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::Connector,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_user(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttUser,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::User,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_user(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttUser,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::User,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_subscribe(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Subscribe,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_subscribe(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::Subscribe,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_topic(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: MQTTTopic,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&topic)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Topic,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_topic(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: MQTTTopic,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&topic)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::Topic,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_node(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&node)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Node,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_node(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&node)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Delete,
        resource_type: BrokerUpdateCacheResourceType::Node,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_resource_config(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    config: ResourceConfig,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&config)?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::ClusterResourceConfig,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}
