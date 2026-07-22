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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::meta::common::call::{get_offset_data, save_offset_data};
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_common::{
    GetOffsetDataRequest, SaveOffsetData, SaveOffsetDataRequest, SaveOffsetDataRequestOffset,
};

pub(crate) fn group_name(queue_name: &str) -> String {
    format!("amqp:{queue_name}")
}

pub(crate) async fn read_committed_offset(
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    queue_name: &str,
    shard_name: &str,
) -> Result<u64, CommonError> {
    let config = broker_config();
    let request = GetOffsetDataRequest {
        tenant: tenant.to_string(),
        group: group_name(queue_name),
    };
    let reply = get_offset_data(client_pool, &config.get_meta_service_addr(), request).await?;
    Ok(reply
        .offsets
        .iter()
        .find(|o| o.shard_name == shard_name)
        .map(|o| o.offset)
        .unwrap_or(0))
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn commit_offset_cas(
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    queue_name: &str,
    shard_name: &str,
    expected: u64,
    new_offset: u64,
) -> Result<bool, CommonError> {
    let config = broker_config();
    let request = SaveOffsetDataRequest {
        offsets: vec![SaveOffsetData {
            tenant: tenant.to_string(),
            group: group_name(queue_name),
            offsets: vec![SaveOffsetDataRequestOffset {
                shard_name: shard_name.to_string(),
                offset: new_offset,
                topic: queue_name.to_string(),
                partition: 0,
                expected_offset: Some(expected),
            }],
        }],
    };
    let reply = save_offset_data(client_pool, &config.get_meta_service_addr(), request).await?;
    Ok(reply.committed)
}
