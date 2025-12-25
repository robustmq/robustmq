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

    let file_size = segment_write.size().await?;
    if let Some(shard_info) = cache_manager.shards.get(&segment_iden.shard_name) {
        if shard_info.last_segment_seq > segment_iden.segment {
            cache_manager.remove_next_segment(&segment_iden.shard_name);
            return Ok(());
        }

        let max_size = shard_info.config.max_segment_size;
        let rate = calc_file_rate(file_size, max_size);
        if rate > SEGMENT_SCROLL_SIZE_THRESHOLD {
            trigger_next_segment_scroll0(
                cache_manager.clone(),
                client_pool.clone(),
                segment_iden.clone(),
                last_offset,
            );
        } else {
            cache_manager.remove_next_segment(&segment_iden.shard_name);
        }
    } else {
        cache_manager.remove_next_segment(&segment_iden.shard_name);
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
        let end_offset = last_offset + SEGMENT_SCROLL_OFFSET_BUFFER;
        let request = CreateNextSegmentRequest {
            shard_name: segment_iden.shard_name.to_string(),
            current_segment: segment_iden.segment as i32,
            current_segment_end_offset: end_offset as i64,
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
                        warn!(
                            "Failed to create next segment for shard '{}', current segment {} (attempt {}/{}): {}. Retrying...",
                            segment_iden.shard_name,
                            segment_iden.segment,
                            attempt,
                            MAX_RETRY_ATTEMPTS,
                            e
                        );
                        sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
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
    use crate::core::test::{test_build_data_fold, test_build_segment, test_init_conf};
    use crate::segment::file::SegmentFile;
    use broker_core::cache::BrokerCacheManager;
    use common_config::broker::default_broker_config;
    use metadata_struct::storage::shard::{
        EngineShard, EngineShardConfig, EngineShardStatus, EngineType,
    };

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
            config: EngineShardConfig {
                replica_num: 1,
                max_segment_size: 1000000,
            },
            engine_type: EngineType::Segment,
            replica_num: 1,
            create_time: 0,
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
