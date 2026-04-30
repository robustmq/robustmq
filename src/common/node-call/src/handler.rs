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

use crate::{UpdateCacheData, RPC_MAX_RETRIES, RPC_RETRY_BASE_MS};
use bytes::Bytes;
use common_base::error::common::CommonError;
use grpc_clients::broker::common::call::{
    broker_get_qos_data_by_client_id, broker_send_last_will_message, broker_update_cache,
};
use grpc_clients::pool::ClientPool;
use prost::Message;
use protocol::broker::broker::{
    GetQosDataByClientIdReply, GetQosDataByClientIdRequest, LastWillClientItem,
    SendLastWillMessageRequest, UpdateCacheRecord, UpdateCacheRequest,
};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::{debug, error, warn};

async fn retry_rpc<F, Fut, R>(addr: &str, label: &str, mut rpc_fn: F)
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<R, CommonError>>,
{
    for attempt in 1..=RPC_MAX_RETRIES {
        match rpc_fn().await {
            Ok(_) => return,
            Err(e) => {
                if attempt >= RPC_MAX_RETRIES {
                    error!(
                        "Failed to {} on broker {} after {} attempts: {}",
                        label, addr, attempt, e
                    );
                    return;
                }
                debug!(
                    "Failed to {} on broker {} (attempt {}/{}): {}, retrying",
                    label, addr, attempt, RPC_MAX_RETRIES, e
                );
                let backoff = RPC_RETRY_BASE_MS.saturating_mul(1u64 << (attempt - 1));
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }
        }
    }
}

pub async fn send_update_cache_batch(
    client_pool: &Arc<ClientPool>,
    addr: &str,
    data: &[UpdateCacheData],
) {
    let records = data
        .iter()
        .map(|raw| UpdateCacheRecord {
            action_type: raw.action_type.into(),
            resource_type: raw.resource_type.into(),
            data: raw.data.clone(),
        })
        .collect();

    let request = UpdateCacheRequest { records };
    let addrs = [addr];

    retry_rpc(addr, "update cache", || {
        broker_update_cache(client_pool, &addrs, request.clone())
    })
    .await;
}

pub async fn send_get_qos_data_batch(
    client_pool: &Arc<ClientPool>,
    addr: &str,
    items: Vec<(String, Option<oneshot::Sender<Bytes>>)>,
) {
    let client_ids: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        items
            .iter()
            .filter(|(id, _)| seen.insert(id.as_str()))
            .map(|(id, _)| id.clone())
            .collect()
    };
    let request = GetQosDataByClientIdRequest { client_ids };
    let addrs = [addr];

    match broker_get_qos_data_by_client_id(client_pool, &addrs, request).await {
        Ok(reply) => {
            // Index the reply by client_id for O(1) lookup per item.
            let index: std::collections::HashMap<&str, _> = reply
                .data
                .iter()
                .map(|raw| (raw.client_id.as_str(), raw))
                .collect();

            for (client_id, reply_tx) in items {
                if let Some(tx) = reply_tx {
                    let data: Vec<_> = index
                        .get(client_id.as_str())
                        .copied()
                        .cloned()
                        .into_iter()
                        .collect();
                    let scoped_reply = GetQosDataByClientIdReply { data };
                    let encoded = Bytes::from(scoped_reply.encode_to_vec());
                    if tx.send(encoded).is_err() {
                        warn!("get_qos_data oneshot receiver dropped before reply was sent");
                    }
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to get_qos_data_by_client_id on broker {}: {}",
                addr, e
            );
        }
    }
}

pub async fn send_last_will_batch(
    client_pool: &Arc<ClientPool>,
    addr: &str,
    items: &[(String, String)],
) {
    let request = SendLastWillMessageRequest {
        items: items
            .iter()
            .map(|(tenant, client_id)| LastWillClientItem {
                tenant: tenant.clone(),
                client_id: client_id.clone(),
            })
            .collect(),
    };
    let addrs = [addr];

    retry_rpc(addr, "send last will messages", || {
        broker_send_last_will_message(client_pool, &addrs, request.clone())
    })
    .await;
}
