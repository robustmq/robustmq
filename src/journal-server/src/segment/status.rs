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

use common_base::config::journal_server::journal_server_conf;
use common_base::tools::now_second;
use dashmap::DashMap;
use grpc_clients::placement::journal::call::{
    create_next_segment, update_segment_meta, update_segment_status,
};
use grpc_clients::pool::ClientPool;
use log::{error, warn};
use metadata_struct::journal::segment::{segment_name, SegmentStatus};
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentRequest, UpdateSegmentMetaRequest, UpdateSegmentStatusRequest,
};
use tokio::time::sleep;

use super::manager::SegmentFileManager;
use super::SegmentIdentity;
use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::write::open_segment_write;

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
        let conf = journal_server_conf();
        loop {
            for segment_iden in self.cache_manager.get_leader_segment() {
                let (segment_write, max_size) = match open_segment_write(
                    self.cache_manager.clone(),
                    &segment_iden.namespace,
                    &segment_iden.shard_name,
                    segment_iden.segment_seq,
                )
                .await
                {
                    Ok((segment_write, max_size)) => (segment_write, max_size),
                    Err(e) => {
                        error!("{}", e);
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
                        error!("{}", e);
                        continue;
                    }
                };

                // 50%
                if self.percentage50_cache.get(&key).is_none() {
                    if file_size / max_size > 50 {
                        let request = CreateNextSegmentRequest {
                            cluster_name: conf.cluster_name.clone(),
                            namespace: segment_iden.namespace.clone(),
                            shard_name: segment_iden.shard_name.clone(),
                        };

                        if let Err(e) = create_next_segment(
                            self.client_pool.clone(),
                            conf.placement_center.clone(),
                            request,
                        )
                        .await
                        {
                            error!("{}", e);
                        } else {
                            self.percentage50_cache.insert(key.clone(), now_second());
                        }
                    }
                }

                // 90%
                if self.percentage50_cache.get(&key).is_none() {
                    if file_size / max_size > 90 {
                        if let Some(current_end_offset) =
                            self.segment_file_manager.get_segment_end_offset(
                                &segment_iden.namespace,
                                &segment_iden.shard_name,
                                segment_iden.segment_seq,
                            )
                        {
                            // update active/next segment status
                            if let Err(e) = segment_status_to_pre_sealup(
                                &self.cache_manager,
                                &self.client_pool,
                                conf.cluster_name.clone(),
                                conf.placement_center.clone(),
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
                            let end_offset = current_end_offset + calc_offset;
                            if let Err(e) = segment_meta_to_pre_sealup(
                                &self.cache_manager,
                                &self.client_pool,
                                conf.cluster_name.clone(),
                                conf.placement_center.clone(),
                                &segment_iden,
                                end_offset,
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
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn calc_end_offset(&self) -> u64 {
        10000
    }
}

pub async fn segment_status_to_sealup(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    namespace: &str,
    shard_name: &str,
    segment_no: u32,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();

    if let Some(segment) = cache_manager.get_segment(namespace, shard_name, segment_no) {
        if segment.status != SegmentStatus::PreSealUp {
            warn!("Segment {} enters the sealup state, but the current state is not PreSealUp, possibly because the Status checking thread is not running.",
            segment_name(namespace, shard_name, segment_no));
        }
        // active segment to sealup
        cache_manager.update_segment_status(
            namespace,
            shard_name,
            segment_no,
            SegmentStatus::SealUp,
        );
        let request = UpdateSegmentStatusRequest {
            cluster_name: conf.cluster_name.clone(),
            namespace: namespace.to_string(),
            shard_name: shard_name.to_string(),
            segment_seq: segment_no,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::SealUp.to_string(),
        };
        update_segment_status(client_pool.clone(), conf.placement_center.clone(), request).await?;
    } else {
        warn!("Segment {} enters the sealup state, but the current Segment is not found, possibly because the Status checking thread is not running.",
        segment_name(namespace, shard_name, segment_no));
    }

    // next segment to Write
    let next_segment_no = segment_no + 1;
    if let Some(segment) = cache_manager.get_segment(namespace, shard_name, next_segment_no) {
        if segment.status != SegmentStatus::PreWrite {
            warn!("segment {} enters the sealup state and the next Segment is not currently in the PreWrite state, possibly because the Status checking thread is not running.",
            segment_name(namespace, shard_name, segment_no));
        }
        // active segment to sealup
        cache_manager.update_segment_status(
            namespace,
            shard_name,
            next_segment_no,
            SegmentStatus::Write,
        );
        let request = UpdateSegmentStatusRequest {
            cluster_name: conf.cluster_name.clone(),
            namespace: namespace.to_string(),
            shard_name: shard_name.to_string(),
            segment_seq: next_segment_no,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::Write.to_string(),
        };
        update_segment_status(client_pool.clone(), conf.placement_center.clone(), request).await?;
    } else {
        warn!("segment {} enters the sealup state, but the next Segment is not found, possibly because the Status checking thread is not running.",
        segment_name(namespace, shard_name, segment_no));
    }
    Ok(())
}

pub async fn segment_status_to_pre_sealup(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    cluster_name: String,
    addrs: Vec<String>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    // active segment to preSealUp
    if let Some(segment) = cache_manager.get_segment(
        &segment_iden.namespace,
        &segment_iden.shard_name,
        segment_iden.segment_seq,
    ) {
        if segment.status != SegmentStatus::Write {
            warn!("Segment {} enters the presealup state, but the current state is not Write, possibly because the Status checking thread is not running.",
            segment_name(&segment_iden.namespace, &segment_iden.shard_name, segment_iden.segment_seq));
        }
        let request = UpdateSegmentStatusRequest {
            cluster_name: cluster_name.clone(),
            namespace: segment_iden.namespace.clone(),
            shard_name: segment_iden.shard_name.clone(),
            segment_seq: segment_iden.segment_seq,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::PreSealUp.to_string(),
        };
        update_segment_status(client_pool.clone(), addrs.clone(), request).await?;
    } else {
        warn!("Segment {} enters the presealup state, but the current Segment is not found, possibly because the Status checking thread is not running.",
        segment_name(&segment_iden.namespace, &segment_iden.shard_name, segment_iden.segment_seq));
    }
    // next segment preWrite
    let next_segment_no = segment_iden.segment_seq + 1;
    if let Some(segment) = cache_manager.get_segment(
        &segment_iden.namespace,
        &segment_iden.shard_name,
        next_segment_no,
    ) {
        if segment.status != SegmentStatus::Write {
            warn!("segment {} enters the presealup state and the next Segment is not currently in the Write state, possibly because the Status checking thread is not running.",
            segment_name(&segment_iden.namespace, &segment_iden.shard_name, segment_iden.segment_seq));
        }
        let request = UpdateSegmentStatusRequest {
            cluster_name: cluster_name.clone(),
            namespace: segment_iden.namespace.clone(),
            shard_name: segment_iden.shard_name.clone(),
            segment_seq: next_segment_no,
            cur_status: segment.status.to_string(),
            next_status: SegmentStatus::PreWrite.to_string(),
        };
        update_segment_status(client_pool.clone(), addrs.clone(), request).await?;
    } else {
        warn!("segment {} enters the presealup state, but the next Segment is not found, possibly because the Status checking thread is not running.",
        segment_name(&segment_iden.namespace, &segment_iden.shard_name, segment_iden.segment_seq));
    }
    Ok(())
}

pub async fn segment_meta_to_pre_sealup(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    cluster_name: String,
    addrs: Vec<String>,
    segment_iden: &SegmentIdentity,
    end_offset: u64,
) -> Result<(), JournalServerError> {
    // update active segment end offset
    let request = UpdateSegmentMetaRequest {
        cluster_name: cluster_name.clone(),
        namespace: segment_iden.namespace.clone(),
        shard_name: segment_iden.shard_name.clone(),
        segment_no: segment_iden.segment_seq,
        start_offset: -1,
        end_offset: end_offset as i64,
        start_timestamp: -1,
        end_timestamp: -1,
    };
    update_segment_meta(client_pool.clone(), addrs.clone(), request).await?;

    // update next segment start offset
    let next_segment_no = segment_iden.segment_seq + 1;
    let request = UpdateSegmentMetaRequest {
        cluster_name: cluster_name.clone(),
        namespace: segment_iden.namespace.clone(),
        shard_name: segment_iden.shard_name.clone(),
        segment_no: next_segment_no,
        start_offset: (end_offset + 1) as i64,
        end_offset: -1,
        start_timestamp: -1,
        end_timestamp: -1,
    };
    update_segment_meta(client_pool.clone(), addrs.clone(), request).await?;
    Ok(())
}
