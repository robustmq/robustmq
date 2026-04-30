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

use crate::core::cache::MQTTCacheManager;
use crate::core::error::MqttBrokerError;
use crate::core::last_will::send_last_will_message;
use crate::storage::last_will::LastWillStorage;
use broker_core::tool::wait_cluster_running;
use grpc_clients::pool::ClientPool;
use protocol::broker::broker::{SendLastWillMessageReply, SendLastWillMessageRequest};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tracing::{debug, warn};

pub async fn send_last_will_message_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    req: &SendLastWillMessageRequest,
) -> Result<SendLastWillMessageReply, MqttBrokerError> {
    wait_cluster_running(&cache_manager.node_cache)
        .await
        .map_err(MqttBrokerError::CommonError)?;

    debug!(
        "Received last will notifications, count: {}",
        req.items.len()
    );

    let last_will_storage = LastWillStorage::new(storage_driver_manager.clone());

    for item in &req.items {
        let data = match last_will_storage
            .get_last_will_message(&item.tenant, &item.client_id)
            .await
        {
            Ok(Some(data)) => data,
            Ok(None) => {
                warn!(
                    "No last will message found for tenant={}, client_id={}",
                    item.tenant, item.client_id
                );
                continue;
            }
            Err(e) => {
                warn!(
                    "Failed to load last will message for tenant={}, client_id={}: {}",
                    item.tenant, item.client_id, e
                );
                continue;
            }
        };

        if let Err(e) =
            send_last_will_message(cache_manager, storage_driver_manager, client_pool, &data).await
        {
            last_will_storage
                .delete_last_will_message(&item.tenant, &item.client_id)
                .await?;
            warn!(
                "Failed to send last will message for tenant={}, client_id={}: {}",
                item.tenant, item.client_id, e
            );
        }
    }

    Ok(SendLastWillMessageReply {})
}
