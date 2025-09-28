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

use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use broker_core::cache::BrokerCacheManager;
use dashmap::DashMap;
use grpc_clients::mqtt::inner::call::broker_mqtt_update_cache;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::placement::node::BrokerNode;
use metadata_struct::resource_config::ClusterResourceConfig;
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
    cluster_name: String,
    data: String,
}

#[derive(Clone)]
pub struct MQTTInnerCallNodeSender {
    sender: Sender<MQTTInnerCallMessage>,
    node: BrokerNode,
}

pub struct MQTTInnerCallManager {
    node_sender: DashMap<String, MQTTInnerCallNodeSender>,
    node_stop_sender: DashMap<String, Sender<bool>>,
    placement_cache_manager: Arc<CacheManager>,
    broker_cache: Arc<BrokerCacheManager>,
}

impl MQTTInnerCallManager {
    pub fn new(
        placement_cache_manager: Arc<CacheManager>,
        broker_cache: Arc<BrokerCacheManager>,
    ) -> Self {
        let node_sender = DashMap::with_capacity(2);
        let node_sender_thread = DashMap::with_capacity(2);
        MQTTInnerCallManager {
            node_sender,
            node_stop_sender: node_sender_thread,
            placement_cache_manager,
            broker_cache,
        }
    }

    pub fn get_node_sender(&self, cluster: &str, node_id: u64) -> Option<MQTTInnerCallNodeSender> {
        let key = self.node_key(cluster, node_id);
        if let Some(sender) = self.node_sender.get(&key) {
            return Some(sender.clone());
        }
        None
    }

    pub fn add_node_sender(&self, cluster: &str, node_id: u64, sender: MQTTInnerCallNodeSender) {
        let key = self.node_key(cluster, node_id);
        self.node_sender.insert(key, sender);
    }

    pub fn remove_node(&self, cluster: &str, node_id: u64) {
        let key = self.node_key(cluster, node_id);
        self.node_sender.remove(&key);
        if let Some((_, send)) = self.node_stop_sender.remove(&key) {
            if let Err(e) = send.send(true) {
                warn!("{}", e);
            }
        }
    }

    pub fn add_node_stop_sender(&self, cluster: &str, node_id: u64, sender: Sender<bool>) {
        let key = self.node_key(cluster, node_id);
        self.node_stop_sender.insert(key, sender);
    }

    fn node_key(&self, cluster: &str, node_id: u64) -> String {
        format!("{cluster}_{node_id}")
    }
}

pub async fn mqtt_call_thread_manager(
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
) {
    loop {
        // start thread
        for (key, node_sender) in call_manager.node_sender.clone() {
            if !call_manager.node_stop_sender.contains_key(&key) {
                let (stop_send, _) = broadcast::channel(2);
                start_call_thread(
                    key.clone(),
                    node_sender.node,
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
                match sx.send(true) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn update_cache_by_add_session(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Session,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_session(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSession,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Session,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_schema(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&schema)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Schema,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_schema(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    schema: SchemaData,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&schema)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Schema,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_schema_bind(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&bind_data)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::SchemaResource,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_schema_bind(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    bind_data: SchemaResourceBind,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&bind_data)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::SchemaResource,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_connector(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MQTTConnector,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Connector,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_connector(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MQTTConnector,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Connector,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_user(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttUser,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::User,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_user(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttUser,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::User,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_subscribe(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Subscribe,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_subscribe(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    session: MqttSubscribe,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&session)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Subscribe,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_topic(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: MQTTTopic,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&topic)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Topic,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_topic(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    topic: MQTTTopic,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&topic)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Topic,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_add_node(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&node)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Node,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_delete_node(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    node: BrokerNode,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&node)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Delete,
        resource_type: MqttBrokerUpdateCacheResourceType::Node,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_resource_config(
    cluster_name: &str,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    config: ClusterResourceConfig,
) -> Result<(), MetaServiceError> {
    let data = serde_json::to_string(&config)?;
    let message = MQTTInnerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::ClusterResourceConfig,
        cluster_name: cluster_name.to_string(),
        data,
    };
    add_call_message(call_manager, cluster_name, client_pool, message).await?;
    Ok(())
}

async fn start_call_thread(
    cluster_name: String,
    node: BrokerNode,
    call_manager: Arc<MQTTInnerCallManager>,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut raw_stop_rx = stop_send.subscribe();
        if let Some(node_send) = call_manager.get_node_sender(&cluster_name, node.node_id) {
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
        let broker_node = match serde_json::from_str::<BrokerNode>(&data.data) {
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
        cluster_name: data.cluster_name.to_string(),
        action_type: data.action_type.into(),
        resource_type: data.resource_type.into(),
        data: data.data.clone(),
    };

    if let Err(e) = broker_mqtt_update_cache(client_pool, &[addr], request.clone()).await {
        if broker_cache.is_stop() {
            return;
        }
        error!("Calling MQTT Broker to update cache failed,{},cluster_name:{},action_type:{},resource_type:{}", e,request.cluster_name,request.action_type,request.resource_type);
    };
}

async fn add_call_message(
    call_manager: &Arc<MQTTInnerCallManager>,
    cluster_name: &str,
    client_pool: &Arc<ClientPool>,
    message: MQTTInnerCallMessage,
) -> Result<(), MetaServiceError> {
    for node in call_manager
        .placement_cache_manager
        .get_broker_node_by_cluster(cluster_name)
    {
        // todo Check whether the node is of the mqtt role
        if let Some(node_sender) = call_manager.get_node_sender(cluster_name, node.node_id) {
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
                cluster_name,
                node.node_id,
                MQTTInnerCallNodeSender {
                    sender: sx.clone(),
                    node: node.clone(),
                },
            );

            // start thread
            let (stop_send, _) = broadcast::channel(2);
            start_call_thread(
                cluster_name.to_string(),
                node.clone(),
                call_manager.clone(),
                client_pool.clone(),
                stop_send.clone(),
            )
            .await;
            call_manager.add_node_stop_sender(cluster_name, node.node_id, stop_send);

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
