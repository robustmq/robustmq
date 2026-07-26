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

pub mod common;
pub mod manager;
pub mod queue;

use std::sync::Arc;

use broker_core::cache::NodeCacheManager;
use broker_core::share_group::ShareGroupStorage;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::share_group::{ShareGroupParams, ShareGroupParamsAmqp};
use network_server::common::connection_manager::ConnectionManager;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::info;

use crate::core::cache::AmqpCacheManager;
pub use manager::AmqpPushManager;
use queue::{AmqpQueuePush, AmqpQueuePushParams};

pub struct PushWatcherParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub amqp_cache: Arc<AmqpCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub push_manager: Arc<AmqpPushManager>,
    pub stop_sx: broadcast::Sender<bool>,
}

/// Every ~200ms, starts/stops a push task per queue this node currently
/// leads (per `NodeCacheManager.share_group_list`) and has consumers for.
/// `Basic.Get` doesn't use this loop — it reads locally or forwards to the
/// leader via `FetchAmqpQueueMessage`.
pub fn start_amqp_push_watcher(params: PushWatcherParams) {
    tokio::spawn(async move {
        let stop_sx = params.stop_sx.clone();
        let ac_fn = async || -> ResultCommonError {
            reconcile(&params).await;
            Ok(())
        };
        loop_select_ticket(ac_fn, 200, &stop_sx).await;
    });
}

async fn reconcile(params: &PushWatcherParams) {
    let self_broker_id = broker_config().broker_id;
    let groups: Vec<_> = params
        .storage_driver_manager
        .broker_cache
        .share_group_list
        .iter()
        .map(|e| e.value().clone())
        .collect();

    for group in groups {
        if !matches!(group.sub_params, ShareGroupParams::AMQP(_)) {
            continue;
        }
        let tenant = group.tenant;
        let queue = group.group_name;
        let members = params
            .storage_driver_manager
            .broker_cache
            .get_share_group_members(&tenant, &queue);
        let is_leader = group.leader_broker == self_broker_id;
        let running = params.push_manager.is_running(&tenant, &queue);

        if is_leader && !members.is_empty() && !running {
            spawn_queue_push(params, tenant, queue);
        } else if (!is_leader || members.is_empty()) && running {
            if let Some(tx) = params.push_manager.mark_stopped(&tenant, &queue) {
                let _ = tx.send(true);
            }
        }
    }
}

fn spawn_queue_push(params: &PushWatcherParams, tenant: String, queue: String) {
    let (stop_tx, _) = broadcast::channel(1);
    params
        .push_manager
        .mark_running(&tenant, &queue, stop_tx.clone());

    let mut push = AmqpQueuePush::new(AmqpQueuePushParams {
        connection_manager: params.connection_manager.clone(),
        storage_driver_manager: params.storage_driver_manager.clone(),
        amqp_cache: params.amqp_cache.clone(),
        client_pool: params.client_pool.clone(),
        push_manager: params.push_manager.clone(),
        tenant: tenant.clone(),
        queue: queue.clone(),
    });
    info!("AMQP queue push task started: {}/{}", tenant, queue);
    tokio::spawn(async move {
        push.start(&stop_tx).await;
    });
}

/// Leader for `queue`'s shared group, creating the group (and electing a
/// leader) on first use — `Basic.Get` has no prior "subscribe" step, so it
/// can't rely on the local cache alone for a queue nobody registered yet.
pub async fn resolve_queue_leader(
    client_pool: &Arc<ClientPool>,
    broker_cache: &Arc<NodeCacheManager>,
    tenant: &str,
    queue: &str,
) -> Result<u64, CommonError> {
    if let Some(group) = broker_cache.get_share_group(tenant, queue) {
        return Ok(group.leader_broker);
    }

    let storage = ShareGroupStorage::new(client_pool.clone());
    if let Some(group) = storage.get(tenant, queue).await? {
        broker_cache.add_share_group(group.clone());
        return Ok(group.leader_broker);
    }

    storage
        .create(
            tenant,
            queue,
            ShareGroupParams::AMQP(ShareGroupParamsAmqp::default()),
        )
        .await?;

    // create and read-back may land on different meta nodes; a follower
    // that hasn't applied the raft entry yet needs a moment to catch up.
    const CREATE_READBACK_RETRIES: u32 = 10;
    const CREATE_READBACK_DELAY_MS: u64 = 30;
    for attempt in 0..CREATE_READBACK_RETRIES {
        if let Some(group) = storage.get(tenant, queue).await? {
            broker_cache.add_share_group(group.clone());
            return Ok(group.leader_broker);
        }
        if attempt + 1 < CREATE_READBACK_RETRIES {
            tokio::time::sleep(std::time::Duration::from_millis(CREATE_READBACK_DELAY_MS)).await;
        }
    }
    Err(CommonError::CommonError(format!(
        "AMQP queue group {tenant}/{queue} still not visible {CREATE_READBACK_RETRIES} reads after being created"
    )))
}

pub fn is_self(broker_id: u64) -> bool {
    broker_id == broker_config().broker_id
}
