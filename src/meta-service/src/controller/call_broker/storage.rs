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

use grpc_clients::pool::ClientPool;
use metadata_struct::journal::{
    segment::JournalSegment, segment_meta::JournalSegmentMetadata, shard::JournalShard,
};
use protocol::broker::broker_common::{BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType};
use std::sync::Arc;

use crate::{
    controller::call_broker::{
        call::add_call_message,
        mqtt::{BrokerCallManager, BrokerCallMessage},
    },
    core::error::MetaServiceError,
};

pub async fn update_cache_by_set_shard(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_info: JournalShard,
) -> Result<(), MetaServiceError> {
    let data = shard_info.encode()?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Shard,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_segment(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment_info: JournalSegment,
) -> Result<(), MetaServiceError> {
    let data = segment_info.encode()?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::Segment,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_segment_meta(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment_info: JournalSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = segment_info.encode()?;
    let message = BrokerCallMessage {
        action_type: BrokerUpdateCacheActionType::Set,
        resource_type: BrokerUpdateCacheResourceType::SegmentMeta,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}
