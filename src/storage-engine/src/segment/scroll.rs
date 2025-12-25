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

use crate::core::error::StorageEngineError;
use crate::segment::SegmentIdentity;
use crate::{core::cache::StorageCacheManager, segment::file::SegmentFile};
use common_config::broker::broker_config;
use grpc_clients::meta::journal::call::create_next_segment;
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_journal::CreateNextSegmentRequest;
use std::sync::Arc;
use tracing::error;

pub fn is_trigger_next_segment_scroll(offsets: &[u64]) -> bool {
    for offset in offsets.iter() {
        if offset % 10000 == 0 {
            return true;
        }
    }

    false
}

pub async fn trigger_next_segment_scroll(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_write: &SegmentFile,
    segment_iden: &SegmentIdentity,
    last_offset: u64,
) -> Result<(), StorageEngineError> {
    if cache_manager
        .is_next_segment
        .contains_key(&segment_iden.shard_name)
    {
        return Ok(());
    }
    let file_size = segment_write.size().await?;
    if let Some(shard_info) = cache_manager.shards.get(&segment_iden.shard_name) {
        if shard_info.last_segment_seq > segment_iden.segment {
            return Ok(());
        }

        let max_size = shard_info.config.max_segment_size;
        let rate = calc_file_rate(file_size, max_size).await;
        if rate > 90 {
            trigger_next_segment_scroll0(
                cache_manager.clone(),
                client_pool.clone(),
                segment_iden.clone(),
                last_offset,
            );
        }
    }
    Ok(())
}

fn trigger_next_segment_scroll0(
    cache_manager: Arc<StorageCacheManager>,
    client_pool: Arc<ClientPool>,
    segment_iden: SegmentIdentity,
    last_offset: u64,
) {
    tokio::spawn(async move {
        cache_manager.add_next_segment(&segment_iden.shard_name, segment_iden.segment);
        let conf = broker_config();
        let calc_offset = calc_end_offset().await;
        let end_offset = last_offset + calc_offset;
        let request = CreateNextSegmentRequest {
            shard_name: segment_iden.shard_name.to_string(),
            current_segment: segment_iden.segment as i32,
            current_segment_end_offset: end_offset as i64,
        };
        if let Err(e) =
            create_next_segment(&client_pool, &conf.get_meta_service_addr(), request).await
        {
            error!("{}", e);
        }
        cache_manager.remove_next_segment(&segment_iden.shard_name);
    });
}

async fn calc_file_rate(file_size: u64, max_size: u64) -> u32 {
    if max_size == 0 {
        return 100;
    }

    ((file_size as f64 / max_size as f64) * 100.0) as u32
}

async fn calc_end_offset() -> u64 {
    10000
}
