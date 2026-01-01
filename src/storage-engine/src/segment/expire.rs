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

use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::{loop_select_ticket, now_second},
};
use common_config::broker::broker_config;
use grpc_clients::{meta::storage::call::delete_segment, pool::ClientPool};
use protocol::meta::meta_service_journal::{DeleteSegmentRaw, DeleteSegmentRequest};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::{core::cache::StorageCacheManager, segment::SegmentIdentity};

pub async fn start_segment_expire_thread(
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<StorageCacheManager>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        scan_and_delete_segment0(&client_pool, &cache_manager).await?;
        Ok(())
    };
    loop_select_ticket(ac_fn, 600000, stop_sx).await;
}

async fn scan_and_delete_segment0(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
) -> Result<(), CommonError> {
    let conf = broker_config();
    let broker_id = conf.broker_id;
    let current_time = now_second();
    let mut segment_list = Vec::new();

    for raw in cache_manager.segments.iter() {
        let shard_name = raw.key();

        let retention_sec = if let Some(shard) = cache_manager.shards.get(shard_name) {
            shard.config.retention_sec
        } else {
            continue;
        };

        let earliest_timestamp = current_time - retention_sec;

        for segment in raw.value().iter() {
            if segment.leader != broker_id {
                continue;
            }

            let segment_iden = SegmentIdentity::new(shard_name, segment.segment_seq);
            let Some(meta) = cache_manager.get_segment_meta(&segment_iden) else {
                continue;
            };

            if (meta.end_timestamp as u64) < earliest_timestamp {
                segment_list.push(DeleteSegmentRaw {
                    shard_name: shard_name.clone(),
                    segment: meta.segment_seq,
                });
            }
        }
    }

    if segment_list.is_empty() {
        return Ok(());
    }

    let request = DeleteSegmentRequest { segment_list };
    delete_segment(client_pool, &conf.get_meta_service_addr(), request).await?;
    Ok(())
}
