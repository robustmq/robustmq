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
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use dashmap::DashMap;
use grpc_clients::journal::inner::call::journal_inner_update_cache;
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::segment::JournalSegment;
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;
use metadata_struct::meta::node::BrokerNode;
use protocol::journal::journal_inner::{
    JournalUpdateCacheActionType, JournalUpdateCacheResourceType, UpdateJournalCacheRequest,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct JournalInnerCallMessage {
    action_type: JournalUpdateCacheActionType,
    resource_type: JournalUpdateCacheResourceType,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct JournalInnerCallNodeSender {
    sender: Sender<JournalInnerCallMessage>,
    node: BrokerNode,
}

pub struct JournalInnerCallManager {
    node_sender: DashMap<u64, JournalInnerCallNodeSender>,
    node_stop_sender: DashMap<u64, Sender<bool>>,
    placement_cache_manager: Arc<CacheManager>,
}

impl JournalInnerCallManager {
    pub fn new(placement_cache_manager: Arc<CacheManager>) -> Self {
        let node_sender = DashMap::with_capacity(2);
        let node_sender_thread = DashMap::with_capacity(2);
        JournalInnerCallManager {
            node_sender,
            node_stop_sender: node_sender_thread,
            placement_cache_manager,
        }
    }

    pub fn get_node_sender(&self, node_id: u64) -> Option<JournalInnerCallNodeSender> {
        if let Some(sender) = self.node_sender.get(&node_id) {
            return Some(sender.clone());
        }
        None
    }

    pub fn add_node_sender(&self, node_id: u64, sender: JournalInnerCallNodeSender) {
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

pub async fn journal_call_thread_manager(
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    stop: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        // start thread
        for (key, node_sender) in call_manager.node_sender.clone() {
            if !call_manager.node_stop_sender.contains_key(&key) {
                let (stop_send, _) = broadcast::channel(2);
                start_call_thread(
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
        Ok(())
    };
    loop_select_ticket(ac_fn, 1000, &stop).await;
}

pub async fn update_cache_by_set_shard(
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_info: JournalShard,
) -> Result<(), MetaServiceError> {
    let data = shard_info.encode()?;
    let message = JournalInnerCallMessage {
        action_type: JournalUpdateCacheActionType::Set,
        resource_type: JournalUpdateCacheResourceType::Shard,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_segment(
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment_info: JournalSegment,
) -> Result<(), MetaServiceError> {
    let data = segment_info.encode()?;
    let message = JournalInnerCallMessage {
        action_type: JournalUpdateCacheActionType::Set,
        resource_type: JournalUpdateCacheResourceType::Segment,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_segment_meta(
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment_info: JournalSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = segment_info.encode()?;
    let message = JournalInnerCallMessage {
        action_type: JournalUpdateCacheActionType::Set,
        resource_type: JournalUpdateCacheResourceType::SegmentMeta,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

async fn start_call_thread(
    node: BrokerNode,
    call_manager: Arc<JournalInnerCallManager>,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut raw_stop_rx = stop_send.subscribe();
        if let Some(node_send) = call_manager.get_node_sender(node.node_id) {
            let mut data_recv = node_send.sender.subscribe();
            info!(
                "Inner communication between Meta Service and Journal Engine node [{:?}].",
                node.node_id
            );
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                info!("Inner communication between Meta Service and Journal Engine node [{:?}].",node.node_id);
                                break;
                            }
                        }
                    },
                    val = data_recv.recv()=>{
                        if let Ok(data) = val{
                            call_journal_update_cache(client_pool.clone(), node.node_inner_addr.clone(), data).await;
                        }
                    }
                }
            }
        }
    });
}

async fn call_journal_update_cache(
    client_pool: Arc<ClientPool>,
    addr: String,
    data: JournalInnerCallMessage,
) {
    let request = UpdateJournalCacheRequest {
        action_type: data.action_type.into(),
        resource_type: data.resource_type.into(),
        data: data.data.clone(),
    };

    match journal_inner_update_cache(&client_pool, &[addr], request).await {
        Ok(resp) => {
            debug!("Calling Journal Engine returns information:{:?}", resp);
        }
        Err(e) => {
            error!("Calling Journal Engine to update cache failed,{}", e);
        }
    };
}

async fn add_call_message(
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    message: JournalInnerCallMessage,
) -> Result<(), MetaServiceError> {
    for node in call_manager.placement_cache_manager.node_list.iter() {
        // todo Check whether the node is of the journal role
        if let Some(node_sender) = call_manager.get_node_sender(node.node_id) {
            match node_sender.sender.send(message.clone()) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        } else {
            // add sender
            let (sx, _) = broadcast::channel::<JournalInnerCallMessage>(1000);
            call_manager.add_node_sender(
                node.node_id,
                JournalInnerCallNodeSender {
                    sender: sx.clone(),
                    node: node.clone(),
                },
            );

            // start thread
            let (stop_send, _) = broadcast::channel(2);
            start_call_thread(
                node.clone(),
                call_manager.clone(),
                client_pool.clone(),
                stop_send.clone(),
            )
            .await;
            call_manager.add_node_stop_sender(node.node_id, stop_send);

            // Wait 2s for the "broadcast rx" thread to start, otherwise the send message will report a "channel closed" error
            sleep(Duration::from_secs(2)).await;

            // send message
            match sx.send(message.clone()) {
                Ok(_) => {}
                Err(e) => {
                    error!("v2{}", e);
                }
            }
        }
    }
    Ok(())
}
