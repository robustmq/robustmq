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

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use grpc_clients::mqtt::inner::call::broker_mqtt_update_cache;
use grpc_clients::pool::ClientPool;
use log::warn;
use log::{debug, error, info};
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MqttTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::placement::node::BrokerNode;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::broker_mqtt::broker_mqtt_inner::MqttBrokerUpdateCacheResourceType;
use protocol::broker_mqtt::broker_mqtt_inner::{
    MqttBrokerUpdateCacheActionType, UpdateMqttCacheRequest,
};

use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;

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
    placement_cache_manager: Arc<PlacementCacheManager>,
}

impl MQTTInnerCallManager {
    pub fn new(placement_cache_manager: Arc<PlacementCacheManager>) -> Self {
        let node_sender = DashMap::with_capacity(2);
        let node_sender_thread = DashMap::with_capacity(2);
        MQTTInnerCallManager {
            node_sender,
            node_stop_sender: node_sender_thread,
            placement_cache_manager,
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
        format!("{}_{}", cluster, node_id)
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
) -> Result<(), PlacementCenterError> {
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
    topic: MqttTopic,
) -> Result<(), PlacementCenterError> {
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
    topic: MqttTopic,
) -> Result<(), PlacementCenterError> {
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
            info!("Thread starts successfully, Inner communication between Placement Center and MQTT Broker node [{:?}].",node);
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                info!("Thread stops successfully, Inner communication between Placement Center and MQTT Broker node [{:?}].",node);
                                break;
                            }
                        }
                    },
                    val = data_recv.recv()=>{
                        if let Ok(data) = val{
                            call_mqtt_update_cache(client_pool.clone(), node.node_inner_addr.clone(), data).await;
                        }
                    }
                }
            }
        }
    });
}

async fn call_mqtt_update_cache(
    client_pool: Arc<ClientPool>,
    addr: String,
    data: MQTTInnerCallMessage,
) {
    let request = UpdateMqttCacheRequest {
        cluster_name: data.cluster_name.to_string(),
        action_type: data.action_type.into(),
        resource_type: data.resource_type.into(),
        data: data.data.clone(),
    };

    match broker_mqtt_update_cache(&client_pool, &[addr], request).await {
        Ok(resp) => {
            debug!("Calling MQTT Broker returns information:{:?}", resp);
        }
        Err(e) => {
            error!("Calling MQTT Broker to update cache failed,{}", e);
        }
    };
}

async fn add_call_message(
    call_manager: &Arc<MQTTInnerCallManager>,
    cluster_name: &str,
    client_pool: &Arc<ClientPool>,
    message: MQTTInnerCallMessage,
) -> Result<(), PlacementCenterError> {
    for node in call_manager
        .placement_cache_manager
        .get_broker_node_by_cluster(cluster_name)
    {
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
