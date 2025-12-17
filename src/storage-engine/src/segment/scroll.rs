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
use std::time::Duration;

use common_base::tools::now_second;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::journal::call::create_next_segment;
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_journal::CreateNextSegmentRequest;
use tokio::time::sleep;
use tracing::{error, info};

use super::file::open_segment_write;
use super::manager::SegmentFileManager;
use crate::core1::cache::CacheManager;
use crate::core1::segment_meta::update_end_and_start_offset;
use crate::core1::segment_status::pre_sealup_segment;

pub struct SegmentScrollManager {
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    segment_file_manager: Arc<SegmentFileManager>,
    percentage50_cache: DashMap<String, u64>,
    percentage90_cache: DashMap<String, u64>,
}

impl SegmentScrollManager {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        client_pool: Arc<ClientPool>,
        segment_file_manager: Arc<SegmentFileManager>,
    ) -> Self {
        let percentage50_cache = DashMap::with_capacity(8);
        let percentage90_cache = DashMap::with_capacity(8);
        SegmentScrollManager {
            cache_manager,
            client_pool,
            percentage50_cache,
            percentage90_cache,
            segment_file_manager,
        }
    }

    pub async fn trigger_segment_scroll(&self) {
        let conf = broker_config();
        info!("Segment scroll thread started successfully");
        loop {
            for segment_iden in self.cache_manager.get_leader_segment() {
                let (segment_write, max_size) =
                    match open_segment_write(&self.cache_manager, &segment_iden).await {
                        Ok((segment_write, max_size)) => (segment_write, max_size),
                        Err(e) => {
                            error!(
                                "Segmen {} File failed to open with error message :{}",
                                segment_iden.name(),
                                e
                            );
                            continue;
                        }
                    };

                let key = segment_iden.name();

                if self.percentage50_cache.contains_key(&key)
                    && self.percentage90_cache.contains_key(&key)
                {
                    continue;
                }

                let file_size = match segment_write.size().await {
                    Ok(size) => size,
                    Err(e) => {
                        error!(
                            "Segmen {} File size calculation failed, error message :{}",
                            segment_iden.name(),
                            e
                        );
                        continue;
                    }
                };

                // create next segment when the file size is greater than 50%
                if self.percentage50_cache.get(&key).is_none() && file_size / max_size as u64 > 50 {
                    let request = CreateNextSegmentRequest {
                        shard_name: segment_iden.shard_name.clone(),
                    };

                    match create_next_segment(
                        &self.client_pool,
                        &conf.get_meta_service_addr(),
                        request,
                    )
                    .await
                    {
                        Ok(_) => {
                            self.percentage50_cache.insert(key.clone(), now_second());
                        }
                        Err(e) => {
                            error!("{}", e);
                            continue;
                        }
                    }
                }

                // 90%
                if self.percentage50_cache.get(&key).is_none() && file_size / max_size as u64 > 98 {
                    if let Some(current_end_offset) =
                        self.segment_file_manager.get_end_offset(&segment_iden)
                    {
                        // update active/next segment status
                        if let Err(e) = pre_sealup_segment(
                            &self.cache_manager,
                            &self.client_pool,
                            &segment_iden,
                        )
                        .await
                        {
                            error!("{}", e);
                            continue;
                        }

                        // update active/next segment end/start offset
                        // calc end_offset
                        let calc_offset = self.calc_end_offset().await;
                        let end_offset = current_end_offset as u64 + calc_offset;
                        if let Err(e) = update_end_and_start_offset(
                            &self.client_pool,
                            &segment_iden,
                            end_offset as i64,
                        )
                        .await
                        {
                            error!("{}", e);
                            continue;
                        }

                        self.percentage90_cache.insert(key.clone(), now_second());
                    } else {
                        error!("When the file size is 90%, try adjusting the segment state. The segment file metadata does not exist, maybe a file is missing.")
                    }
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn calc_end_offset(&self) -> u64 {
        // todo
        10000
    }
}
