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

use crate::controller::call_broker::mqtt::BrokerCallManager;
use crate::controller::call_broker::mqtt::BrokerCallMessage;
use crate::controller::call_broker::mqtt::BrokerCallNodeSender;
use crate::core::error::MetaServiceError;
use broker_core::cache::BrokerCacheManager;
use common_base::utils::serialize;
use grpc_clients::broker::common::call::broker_common_update_cache;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use protocol::broker::broker_common::BrokerUpdateCacheResourceType;
use protocol::broker::broker_common::UpdateCacheRequest;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::{self};
use tokio::time::sleep;
use tracing::error;
use tracing::info;

pub async fn start_call_thread(
    node: BrokerNode,
    call_manager: Arc<BrokerCallManager>,
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

fn is_ignore_push(node: &BrokerNode, data: &BrokerCallMessage) -> bool {
    if data.resource_type == BrokerUpdateCacheResourceType::Node {
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
    data: &BrokerCallMessage,
) {
    let request = UpdateCacheRequest {
        action_type: data.action_type.into(),
        resource_type: data.resource_type.into(),
        data: data.data.clone(),
    };

    if let Err(e) = broker_common_update_cache(client_pool, &[addr], request.clone()).await {
        if broker_cache.is_stop().await {
            return;
        }
        error!(
            "Calling MQTT Broker to update cache failed,{},action_type:{:?},resource_type:{:?},addr:{}",
            e, request.action_type(), request.resource_type(), addr
        );
    };
}

pub async fn add_call_message(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    message: BrokerCallMessage,
) -> Result<(), MetaServiceError> {
    for raw in call_manager.broker_cache.node_list().iter() {
        // todo Check whether the node is of the mqtt role
        if let Some(node_sender) = call_manager.get_node_sender(raw.node_id) {
            match node_sender.sender.send(message.clone()) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        } else {
            // add sender
            let (sx, _) = broadcast::channel::<BrokerCallMessage>(1000);
            call_manager.add_node_sender(
                raw.node_id,
                BrokerCallNodeSender {
                    sender: sx.clone(),
                    node: raw.clone(),
                },
            );

            // start thread
            let (stop_send, _) = broadcast::channel(2);
            start_call_thread(
                raw.clone(),
                call_manager.clone(),
                client_pool.clone(),
                stop_send.clone(),
            )
            .await;
            call_manager.add_node_stop_sender(raw.node_id, stop_send);

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
