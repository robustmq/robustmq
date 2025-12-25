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

use super::error::StorageEngineError;
use crate::segment::SegmentIdentity;
use common_config::broker::broker_config;
use grpc_clients::meta::journal::call::update_segment_meta;
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_journal::UpdateSegmentMetaRequest;
use std::sync::Arc;

pub async fn update_meta_start_timestamp(
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    start_timestamp: u64,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();
    let next_segment_no = segment_iden.segment;
    let request = UpdateSegmentMetaRequest {
        shard_name: segment_iden.shard_name.clone(),
        segment_no: next_segment_no,
        start_offset: -1,
        end_offset: -1,
        start_timestamp: start_timestamp as i64,
        end_timestamp: -1,
    };
    update_segment_meta(client_pool, &conf.get_meta_service_addr(), request).await?;
    Ok(())
}

pub async fn update_meta_end_timestamp(
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    end_timestamp: u64,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();

    let request = UpdateSegmentMetaRequest {
        shard_name: segment_iden.shard_name.clone(),
        segment_no: segment_iden.segment,
        start_offset: -1,
        end_offset: -1,
        start_timestamp: -1,
        end_timestamp: end_timestamp as i64,
    };
    update_segment_meta(client_pool, &conf.get_meta_service_addr(), request).await?;

    Ok(())
}
