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

use common_config::journal::config::journal_server_conf;
use grpc_clients::placement::journal::call::update_segment_meta;
use grpc_clients::pool::ClientPool;
use protocol::placement_center::placement_center_journal::UpdateSegmentMetaRequest;
use tracing::warn;

use super::error::JournalServerError;
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

pub async fn update_end_and_start_offset(
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    end_offset: i64,
) -> Result<(), JournalServerError> {
    // update active segment end offset
    update_meta_end_offset(client_pool.clone(), segment_iden, end_offset).await?;

    // update next segment start offset
    let mut new_segment_iden = segment_iden.clone();
    new_segment_iden.segment_seq = segment_iden.segment_seq + 1;

    update_meta_start_offset(client_pool.clone(), segment_iden, end_offset + 1).await?;
    Ok(())
}

pub async fn update_meta_start_timestamp(
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    start_timestamp: u64,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();
    let next_segment_no = segment_iden.segment_seq;
    let request = UpdateSegmentMetaRequest {
        cluster_name: conf.cluster_name.clone(),
        namespace: segment_iden.namespace.clone(),
        shard_name: segment_iden.shard_name.clone(),
        segment_no: next_segment_no,
        start_offset: -1,
        end_offset: -1,
        start_timestamp: start_timestamp as i64,
        end_timestamp: -1,
    };
    update_segment_meta(client_pool, &conf.placement_center, request).await?;
    Ok(())
}

pub async fn update_meta_end_timestamp(
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    segment_file_manager: &Arc<SegmentFileManager>,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();
    if let Some(file) = segment_file_manager.get_segment_file(segment_iden) {
        let next_segment_no = segment_iden.segment_seq;

        if file.end_timestamp > 0 {
            let request = UpdateSegmentMetaRequest {
                cluster_name: conf.cluster_name.clone(),
                namespace: segment_iden.namespace.clone(),
                shard_name: segment_iden.shard_name.clone(),
                segment_no: next_segment_no,
                start_offset: -1,
                end_offset: -1,
                start_timestamp: -1,
                end_timestamp: file.end_timestamp,
            };
            update_segment_meta(client_pool, &conf.placement_center, request).await?;
        } else {
            warn!("");
        }
    }
    Ok(())
}

async fn update_meta_start_offset(
    client_pool: Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    start_offset: i64,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();
    let next_segment_no = segment_iden.segment_seq;
    let request = UpdateSegmentMetaRequest {
        cluster_name: conf.cluster_name.clone(),
        namespace: segment_iden.namespace.clone(),
        shard_name: segment_iden.shard_name.clone(),
        segment_no: next_segment_no,
        start_offset,
        end_offset: -1,
        start_timestamp: -1,
        end_timestamp: -1,
    };
    update_segment_meta(&client_pool, &conf.placement_center, request).await?;
    Ok(())
}

async fn update_meta_end_offset(
    client_pool: Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    end_offset: i64,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();
    let next_segment_no = segment_iden.segment_seq;
    let request = UpdateSegmentMetaRequest {
        cluster_name: conf.cluster_name.clone(),
        namespace: segment_iden.namespace.clone(),
        shard_name: segment_iden.shard_name.clone(),
        segment_no: next_segment_no,
        start_offset: -1,
        end_offset,
        start_timestamp: -1,
        end_timestamp: -1,
    };
    update_segment_meta(&client_pool, &conf.placement_center, request).await?;
    Ok(())
}
