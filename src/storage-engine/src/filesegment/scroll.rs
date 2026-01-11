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
use crate::filesegment::SegmentIdentity;
use crate::{core::cache::StorageCacheManager, filesegment::segment_file::SegmentFile};
use common_base::tools::now_second;
use common_config::broker::broker_config;
use grpc_clients::meta::storage::call::{
    create_next_segment, seal_up_segment, update_start_time_by_segment_meta,
};
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentRequest, SealUpSegmentRequest, UpdateStartTimeBySegmentMetaRequest,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, warn};

const SEGMENT_SCROLL_OFFSET_INTERVAL: u64 = 10000;
const SEGMENT_SCROLL_SIZE_THRESHOLD: u32 = 90;
const SEGMENT_SCROLL_OFFSET_BUFFER: u64 = 10000;
const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

pub fn is_trigger_next_segment_scroll(offsets: &[u64]) -> bool {
    offsets
        .last()
        .is_some_and(|&offset| offset % SEGMENT_SCROLL_OFFSET_INTERVAL == 0)
}

pub fn is_start_or_end_offset(
    cache_manager: &Arc<StorageCacheManager>,
    segment_iden: &SegmentIdentity,
    offsets: &[u64],
) -> bool {
    if let Some(meta) = cache_manager.get_segment_meta(segment_iden) {
        let start_offset = meta.start_offset.max(0) as u64;
        let end_offset = meta.end_offset.max(0) as u64;
        return offsets.contains(&start_offset) || offsets.contains(&end_offset);
    }
    warn!(
        "Segment metadata not found for shard '{}' segment {}, expected to exist",
        segment_iden.shard_name, segment_iden.segment
    );
    false
}

pub fn trigger_update_start_or_end_info(
    cache_manager: Arc<StorageCacheManager>,
    client_pool: Arc<ClientPool>,
    segment_iden: SegmentIdentity,
    offsets: Vec<u64>,
) {
    tokio::spawn(async move {
        if let Err(e) =
            trigger_update_start_or_end_info0(&cache_manager, &client_pool, &segment_iden, &offsets)
                .await
        {
            error!(
                "Failed to update start/end timestamp for shard '{}' segment {}: {}",
                segment_iden.shard_name, segment_iden.segment, e
            );
        };
    });
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
        .insert(segment_iden.shard_name.clone(), segment_iden.segment)
        .is_some()
    {
        return Ok(());
    }

    let should_trigger =
        if let Some(shard_info) = cache_manager.shards.get(&segment_iden.shard_name) {
            if shard_info.last_segment_seq > segment_iden.segment {
                false
            } else {
                let file_size = segment_write.size().await?;
                let max_size = shard_info.config.max_segment_size;
                let rate = calc_file_rate(file_size, max_size);
                rate > SEGMENT_SCROLL_SIZE_THRESHOLD
            }
        } else {
            false
        };

    if should_trigger {
        trigger_next_segment_scroll0(
            cache_manager.clone(),
            client_pool.clone(),
            segment_iden.clone(),
            last_offset,
        );
    } else {
        cache_manager.remove_next_segment(&segment_iden.shard_name);
    }

    Ok(())
}

async fn trigger_update_start_or_end_info0(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    offsets: &[u64],
) -> Result<(), StorageEngineError> {
    if let Some(meta) = cache_manager.get_segment_meta(segment_iden) {
        let start_offset = meta.start_offset.max(0) as u64;
        let end_offset = meta.end_offset.max(0) as u64;
        let is_start = offsets.contains(&start_offset);
        let is_end = offsets.contains(&end_offset);

        if is_start || is_end {
            let conf = broker_config();

            if is_start {
                let request = UpdateStartTimeBySegmentMetaRequest {
                    shard_name: segment_iden.shard_name.clone(),
                    segment: segment_iden.segment,
                    start_timestamp: now_second(),
                };
                update_start_time_by_segment_meta(
                    client_pool,
                    &conf.get_meta_service_addr(),
                    request,
                )
                .await?;
            }

            if is_end {
                let request = SealUpSegmentRequest {
                    shard_name: segment_iden.shard_name.clone(),
                    segment: segment_iden.segment,
                    end_timestamp: now_second(),
                };
                seal_up_segment(client_pool, &conf.get_meta_service_addr(), request).await?;
            }
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
        let conf = broker_config();
        let end_offset = last_offset.saturating_add(SEGMENT_SCROLL_OFFSET_BUFFER);

        let current_segment = segment_iden.segment.min(i32::MAX as u32) as i32;
        let end_offset_i64 = end_offset.min(i64::MAX as u64) as i64;

        let request = CreateNextSegmentRequest {
            shard_name: segment_iden.shard_name.clone(),
            current_segment,
            current_segment_end_offset: end_offset_i64,
        };

        for attempt in 1..=MAX_RETRY_ATTEMPTS {
            match create_next_segment(&client_pool, &conf.get_meta_service_addr(), request.clone())
                .await
            {
                Ok(_) => {
                    break;
                }
                Err(e) => {
                    if attempt == MAX_RETRY_ATTEMPTS {
                        error!(
                            "Failed to create next segment for shard '{}', current segment {}, end offset {} after {} attempts: {}",
                            segment_iden.shard_name,
                            segment_iden.segment,
                            end_offset,
                            MAX_RETRY_ATTEMPTS,
                            e
                        );
                    } else {
                        let delay = RETRY_DELAY_MS * (1 << (attempt - 1));
                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        cache_manager.remove_next_segment(&segment_iden.shard_name);
    });
}

fn calc_file_rate(file_size: u64, max_size: u64) -> u32 {
    if max_size == 0 {
        return 100;
    }

    ((file_size as f64 / max_size as f64) * 100.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cache::StorageCacheManager;
    use crate::core::test_tool::{test_build_data_fold, test_build_segment, test_init_conf};
    use crate::filesegment::segment_file::SegmentFile;
    use broker_core::cache::BrokerCacheManager;
    use common_config::broker::default_broker_config;
    use metadata_struct::storage::shard::{EngineShard, EngineShardConfig, EngineShardStatus};

    #[test]
    fn is_trigger_scroll_test() {
        assert!(!is_trigger_next_segment_scroll(&[]));
        assert!(is_trigger_next_segment_scroll(&[9999, 10000]));
        assert!(is_trigger_next_segment_scroll(&[20000]));
        assert!(!is_trigger_next_segment_scroll(&[10000, 20000, 9999]));
    }

    #[test]
    fn calc_file_rate_test() {
        assert_eq!(calc_file_rate(1000, 0), 100);
        assert_eq!(calc_file_rate(0, 1000), 0);
        assert_eq!(calc_file_rate(900, 1000), 90);
        assert_eq!(calc_file_rate(1000, 1000), 100);
    }

    #[tokio::test]
    async fn scroll_already_triggered_test() {
        test_init_conf();
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();
        let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(BrokerCacheManager::new(
            default_broker_config(),
        ))));
        let client_pool = Arc::new(ClientPool::new(10));

        cache_manager.add_next_segment(&segment_iden.shard_name, segment_iden.segment);

        let segment_file = SegmentFile::new(
            segment_iden.shard_name.clone(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        let result = trigger_next_segment_scroll(
            &cache_manager,
            &client_pool,
            &segment_file,
            &segment_iden,
            100,
        )
        .await;

        assert!(result.is_ok());
        assert!(cache_manager
            .is_next_segment
            .contains_key(&segment_iden.shard_name));
    }

    #[tokio::test]
    async fn scroll_not_trigger_test() {
        test_init_conf();
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();

        let cache_manager = Arc::new(StorageCacheManager::new(Arc::new(BrokerCacheManager::new(
            default_broker_config(),
        ))));
        let client_pool = Arc::new(ClientPool::new(10));

        let shard = EngineShard {
            shard_uid: "test-shard-uid".to_string(),
            shard_name: segment_iden.shard_name.clone(),
            start_segment_seq: segment_iden.segment,
            active_segment_seq: segment_iden.segment,
            last_segment_seq: segment_iden.segment,
            status: EngineShardStatus::Run,
            config: EngineShardConfig::default(),
            create_time: now_second(),
        };
        cache_manager.set_shard(shard);

        let segment_file = SegmentFile::new(
            segment_iden.shard_name.clone(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();
        segment_file.try_create().await.unwrap();

        let result = trigger_next_segment_scroll(
            &cache_manager,
            &client_pool,
            &segment_file,
            &segment_iden,
            100,
        )
        .await;

        assert!(result.is_ok());
        assert!(!cache_manager
            .is_next_segment
            .contains_key(&segment_iden.shard_name));
    }
}
