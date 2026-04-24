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

use crate::core::error::NatsBrokerError;
use broker_core::share_group::ShareGroupStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::nats::subscriber::NatsSubscriber;
use std::sync::Arc;

pub async fn add_member_by_group(
    client_pool: Arc<ClientPool>,
    sub: &NatsSubscriber,
) -> Result<(), NatsBrokerError> {
    let storage = ShareGroupStorage::new(client_pool);
    storage
        .add_member(&sub.tenant, &sub.queue_group, sub)
        .await?;
    Ok(())
}

pub async fn delete_member_by_group(
    client_pool: Arc<ClientPool>,
    sub: &NatsSubscriber,
) -> Result<(), NatsBrokerError> {
    let storage = ShareGroupStorage::new(client_pool);
    storage
        .delete_member(&sub.tenant, &sub.queue_group, sub)
        .await?;
    Ok(())
}

pub async fn delete_members_by_group(client_pool: &Arc<ClientPool>, subs: Vec<NatsSubscriber>) {
    for sub in &subs {
        if let Err(e) = delete_member_by_group(client_pool.clone(), sub).await {
            tracing::error!(
                "Failed to delete share group member [tenant={}, group={}, sid={}]: {}",
                sub.tenant,
                sub.queue_group,
                sub.sid,
                e
            );
        }
    }
}
