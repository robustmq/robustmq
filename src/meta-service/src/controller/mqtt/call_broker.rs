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

use crate::core::error::MetaServiceError;
use broker_core::cache::BrokerCacheManager;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_base::utils::serialize;
use dashmap::DashMap;
use grpc_clients::mqtt::inner::call::broker_mqtt_update_cache;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::resource_config::ResourceConfig;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker::broker_mqtt_inner::MqttBrokerUpdateCacheResourceType;
use protocol::broker::broker_mqtt_inner::{
    MqttBrokerUpdateCacheActionType, UpdateMqttCacheRequest,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
use tracing::error;
use tracing::info;
use tracing::warn;

#[derive(Clone)]
pub struct MQTTInnerCallMessage {
    action_type: MqttBrokerUpdateCacheActionType,
    resource_type: MqttBrokerUpdateCacheResourceType,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct MQTTInnerCallNodeSender {
    sender: Sender<MQTTInnerCallMessage>,
    node: BrokerNode,
}

pub struct MQTTInnerCallManager {
    node_sender: DashMap<u64, MQTTInnerCallNodeSender>,
    node_stop_sender: DashMap<u64, Sender<bool>>,
    broker_cache: Arc<BrokerCacheManager>,
}

impl MQTTInnerCallManager {
    pub fn new(broker_cache: Arc<BrokerCacheManager>) -> Self {
        let node_sender = DashMap::with_capacity(2);
        let node_stop_sender = DashMap::with_capacity(2);
        MQTTInnerCallManager {
            node_sender,
            node_stop_sender,
            broker_cache,
        }
    }

    pub fn get_node_sender(&self, node_id: u64) -> Option<MQTTInnerCallNodeSender> {
        if let Some(sender) = self.node_sender.get(&node_id) {
            return Some(sender.clone());
        }
        None
    }

    pub fn add_node_sender(&self, node_id: u64, sender: MQTTInnerCallNodeSender) {
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

pub async fn mqtt_call_thread_manager(
    call_manager: &Arc<MQTTInnerCallManager>,
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
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Session,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_session(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Session,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_schema(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&schema)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Schema,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_schema(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&schema)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Schema,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_schema_bind(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&bind_data)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::SchemaResource,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_schema_bind(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&bind_data)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::SchemaResource,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_connector(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MQTTConnector,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Connector,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_connector(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MQTTConnector,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Connector,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_user(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttUser,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::User,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_user(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttUser,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::User,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_subscribe(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Subscribe,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_subscribe(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Subscribe,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_topic(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: MQTTTopic,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&topic)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Topic,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_topic(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: MQTTTopic,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&topic)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Topic,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_node(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&node)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Node,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_node(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&node)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Node,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_resource_config(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    config: ResourceConfig,
) -> Result<(), MetaServiceError> {
    let data = serialize::serialize(&config)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::ClusterResourceConfig,

        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

async fn start_call_thread(
    node: BrokerNode,
    call_manager: Arc<MQTTInnerCallManager>,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut raw_stop_rx = stop_send.subscribe();
        if let Some(node_send) = call_manager.get_node_sender(node.node_id) {
            let mut data_recv = node_send.sender.subscribe();
            info!(
                "Inner communication between Meta Service and MQTT Broker node [{:?}].",
                node.node_id
            );
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                info!("Inner communication between Meta Service and MQTT Broker node [{:?}].",node.node_id);
                                break;
                            }
                        }
                    },
                    val = data_recv.recv()=>{
                        if let Ok(data) = val{
                            if is_ignore_push(&node, &data){
                                continue;
                            }
                            call_mqtt_update_cache(&client_pool, &call_manager.broker_cache, &node.node_inner_addr, &data).await;
                        }
                    }
                }
            }
        }
    });
}

fn is_ignore_push(node: &BrokerNode, data: &MQTTInnerCallMessage) -> bool {
    if data.resource_type == MqttBrokerUpdateCacheResourceType::Node {
        let broker_node = match serialize::deserialize::<BrokerNode>(&data.data) {
            Ok(node) => node,
            Err(_) => {
                return true;
            }
        };
        return broker_node.node_id == node.node_id;
    }
    false
}
async fn call_mqtt_update_cache(
    client_pool: &Arc<ClientPool>,
    broker_cache: &Arc<BrokerCacheManager>,
    addr: &String,
    data: &MQTTInnerCallMessage,
) {
    let request = UpdateMqttCacheRequest {
        action_type: data.action_type.into(),
        resource_type: data.resource_type.into(),
        data: data.data.clone(),
    };

    if let Err(e) = broker_mqtt_update_cache(client_pool, &[addr], request.clone()).await {
        if broker_cache.is_stop().await {
            return;
        }
        error!(
            "Calling MQTT Broker to update cache failed,{},action_type:{},resource_type:{}",
            e, request.action_type, request.resource_type
        );
    };
}

async fn add_call_message(
    call_manager: &Arc<MQTTInnerCallManager>,

    client_pool: &Arc<ClientPool>,
    message: MQTTInnerCallMessage,
) -> Result<(), MetaServiceError> {
    for raw in call_manager.node_sender.iter() {
        // todo Check whether the node is of the mqtt role
        if let Some(node_sender) = call_manager.get_node_sender(raw.node.node_id) {
            match node_sender.sender.send(message.clone()) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        } else {
            // add sender
            let (sx, _) = broadcast::channel::<MQTTInnerCallMessage>(1000);
            call_manager.add_node_sender(
                raw.node.node_id,
                MQTTInnerCallNodeSender {
                    sender: sx.clone(),
                    node: raw.node.clone(),
                },
            );

            // start thread
            let (stop_send, _) = broadcast::channel(2);
            start_call_thread(
                raw.node.clone(),
                call_manager.clone(),
                client_pool.clone(),
                stop_send.clone(),
            )
            .await;
            call_manager.add_node_stop_sender(raw.node.node_id, stop_send);

            // Wait 2s for the "broadcast rx" thread to start, otherwise the send message will report a "channel closed" error
            sleep(Duration::from_secs(2)).await;

            // send message
            match sx.send(message.clone()) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
    Ok(())
}
