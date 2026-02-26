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
use common_base::error::common::CommonError;
use grpc_clients::broker::common::call::broker_common_update_cache;
use grpc_clients::broker::mqtt::call::{broker_mqtt_delete_session, send_last_will_message};
use grpc_clients::pool::ClientPool;
use protocol::broker::broker_common::{UpdateCacheRecord, UpdateCacheRequest};
use protocol::broker::broker_mqtt::{
    DeleteSessionRequest, LastWillMessageItem, SendLastWillMessageRequest,
};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, warn};

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
                warn!(
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
        broker_common_update_cache(client_pool, &addrs, request.clone())
    })
    .await;
}

pub async fn send_delete_session_batch(
    client_pool: &Arc<ClientPool>,
    addr: &str,
    client_ids: &[String],
) {
    let request = DeleteSessionRequest {
        client_id: client_ids.to_vec(),
    };
    let addrs = [addr];

    retry_rpc(addr, "delete sessions", || {
        broker_mqtt_delete_session(client_pool, &addrs, request.clone())
    })
    .await;
}

pub async fn send_last_will_batch(
    client_pool: &Arc<ClientPool>,
    addr: &str,
    items: &[LastWillMessageItem],
) {
    let request = SendLastWillMessageRequest {
        items: items.to_vec(),
    };
    let addrs = [addr];

    retry_rpc(addr, "send last will messages", || {
        send_last_will_message(client_pool, &addrs, request.clone())
    })
    .await;
}
