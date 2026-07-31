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

use broker_core::share_group::ShareGroupStorage;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::share_group::{
    ShareGroupMember, ShareGroupParams, ShareGroupParamsAmqp,
};

// AMQP has no separate "queue group name" — the queue itself is the
// competing-consumer boundary, so group_name == queue_name.
fn sid(channel_id: u16, consumer_tag: &str) -> String {
    format!("{channel_id}:{consumer_tag}")
}

/// Registers one `Basic.Consume` as a member of its queue's shared group.
/// Meta-service auto-creates the group (electing a leader) on first member.
#[allow(clippy::too_many_arguments)]
pub async fn add_consume_member(
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    queue: &str,
    connect_id: u64,
    broker_id: u64,
    channel_id: u16,
    consumer_tag: &str,
    no_ack: bool,
    exclusive: bool,
) -> Result<(), CommonError> {
    let storage = ShareGroupStorage::new(client_pool.clone());
    let member = ShareGroupMember {
        tenant: tenant.to_string(),
        group_name: queue.to_string(),
        broker_id,
        sub_path: queue.to_string(),
        sid: sid(channel_id, consumer_tag),
        params: ShareGroupParams::AMQP(ShareGroupParamsAmqp {
            channel_id,
            consumer_tag: consumer_tag.to_string(),
            no_ack,
            exclusive,
        }),
        connect_id,
        create_time: now_second(),
    };
    storage.add_member(&member).await
}

pub async fn remove_consume_member(
    client_pool: &Arc<ClientPool>,
    broker_id: u64,
    connect_id: u64,
    channel_id: u16,
    consumer_tag: &str,
) -> Result<(), CommonError> {
    let storage = ShareGroupStorage::new(client_pool.clone());
    storage
        .delete_member(broker_id, connect_id, &sid(channel_id, consumer_tag))
        .await
}
