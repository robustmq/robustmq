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

use common_config::broker::broker_config;
use grpc_clients::meta::storage::call::update_segment_status;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::SegmentStatus;
use protocol::meta::meta_service_journal::UpdateSegmentStatusRequest;
use tracing::warn;

use super::cache::StorageCacheManager;
use super::error::StorageEngineError;
use crate::segment::SegmentIdentity;

pub async fn pre_sealup_segment(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    // active segment to preSealUp
    update_segment_status_to_pre_seal_up(cache_manager, client_pool, segment_iden).await?;

    // next segment preWrite
    let mut next_segment_iden = segment_iden.clone();
    next_segment_iden.segment = segment_iden.segment + 1;
    update_segment_status_to_pre_write(cache_manager, client_pool, &next_segment_iden).await
}

pub async fn sealup_segment(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    // active segment to sealUp
    update_segment_status_to_seal_up(cache_manager, client_pool, segment_iden).await?;

    // next segment to Write
    let mut next_segment_iden = segment_iden.clone();
    next_segment_iden.segment = segment_iden.segment + 1;
    update_segment_status_to_write(cache_manager, client_pool, &next_segment_iden).await
}

async fn update_segment_status_to_pre_write(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();
    if let Some(segment) = cache_manager.get_segment(segment_iden) {
        if segment.status != SegmentStatus::Idle {
            warn!("Segment {} enters the PreWrite state, but the current state is not Idle, possibly because the Status checking thread is not running.",
            segment_iden.name());
            return Ok(());
        }

        // update cache status
        cache_manager.update_segment_status(segment_iden, SegmentStatus::PreWrite);

        // update meta status
        let request = UpdateSegmentStatusRequest {
            shard_name: segment_iden.shard_name.to_string(),
            segment_seq: segment_iden.segment,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::PreWrite.to_string(),
        };
        update_segment_status(client_pool, &conf.get_meta_service_addr(), request).await?;
    }
    Ok(())
}

async fn update_segment_status_to_write(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();
    if let Some(segment) = cache_manager.get_segment(segment_iden) {
        if segment.status != SegmentStatus::PreWrite {
            warn!("segment {} enters the sealup state and the next Segment is not currently in the PreWrite state, possibly because the Status checking thread is not running.",
            segment_iden.name());
        }
        // update cache status
        cache_manager.update_segment_status(segment_iden, SegmentStatus::Write);

        // update meta status
        let request = UpdateSegmentStatusRequest {
            shard_name: segment_iden.shard_name.to_string(),
            segment_seq: segment_iden.segment,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::Write.to_string(),
        };
        update_segment_status(client_pool, &conf.get_meta_service_addr(), request).await?;
    }
    Ok(())
}

async fn update_segment_status_to_pre_seal_up(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();
    if let Some(segment) = cache_manager.get_segment(segment_iden) {
        if segment.status != SegmentStatus::Write {
            warn!("Segment {} enters the PreSealup state, but the current state is not Write, possibly because the Status checking thread is not running.",
            segment_iden.name());
            return Ok(());
        }

        // update cache status
        cache_manager.update_segment_status(segment_iden, SegmentStatus::PreSealUp);

        // update meta status
        let request = UpdateSegmentStatusRequest {
            shard_name: segment_iden.shard_name.to_string(),
            segment_seq: segment_iden.segment,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::PreSealUp.to_string(),
        };
        update_segment_status(client_pool, &conf.get_meta_service_addr(), request).await?;
    }

    Ok(())
}

async fn update_segment_status_to_seal_up(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();
    if let Some(segment) = cache_manager.get_segment(segment_iden) {
        if segment.status != SegmentStatus::PreSealUp {
            warn!("Segment {} enters the sealup state, but the current state is not PreSealUp, possibly because the Status checking thread is not running.",
            segment_iden.name());
            return Ok(());
        }

        // update cache status
        cache_manager.update_segment_status(segment_iden, SegmentStatus::SealUp);

        // update meta status
        let request = UpdateSegmentStatusRequest {
            shard_name: segment_iden.shard_name.to_string(),
            segment_seq: segment_iden.segment,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::SealUp.to_string(),
        };
        update_segment_status(client_pool, &conf.get_meta_service_addr(), request).await?;
    }
    Ok(())
}
