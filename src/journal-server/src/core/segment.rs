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

use common_base::config::journal_server::journal_server_conf;
use log::error;
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileRequest, GetSegmentDeleteStatusRequest,
};
use rocksdb_engine::RocksDBEngine;

use super::cache::CacheManager;
use super::error::JournalServerError;
use crate::index::build::delete_segment_index;
use crate::segment::file::{open_segment_write, SegmentFile};
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

pub fn delete_local_segment(
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_file_manager: Arc<SegmentFileManager>,
    req: DeleteSegmentFileRequest,
) -> Result<(), JournalServerError> {
    let segment_iden = SegmentIdentity::new(&req.namespace, &req.shard_name, req.segment);

    let segment = if let Some(segment) = cache_manager.get_segment(&segment_iden) {
        segment
    } else {
        return Err(JournalServerError::SegmentNotExist(segment_iden.name()));
    };

    tokio::spawn(async move {
        // delete local file
        let conf = journal_server_conf();
        let data_fold = if let Some(fold) = segment.get_fold(conf.node_id) {
            fold
        } else {
            return;
        };

        let segment_file = SegmentFile::new(
            segment_iden.namespace.clone(),
            segment_iden.shard_name.clone(),
            segment_iden.segment_seq,
            data_fold,
        );
        if let Err(e) = segment_file.delete().await {
            error!("{}", e);
        }

        // delete index
        if let Err(e) = delete_segment_index(&rocksdb_engine_handler, &segment_iden) {
            error!("{}", e);
        }

        // delete segment
        cache_manager.delete_segment(&segment_iden);

        // delete segment file manager
        segment_file_manager.remove_segment_file(&segment_iden);
    });
    Ok(())
}

pub async fn segment_already_delete(
    cache_manager: &Arc<CacheManager>,
    req: &GetSegmentDeleteStatusRequest,
) -> Result<bool, JournalServerError> {
    let segment_iden = SegmentIdentity {
        namespace: req.namespace.clone(),
        shard_name: req.shard_name.clone(),
        segment_seq: req.segment,
    };

    // Does the file exist
    let (segment_write, _) = open_segment_write(cache_manager, &segment_iden).await?;

    Ok(!segment_write.exists())
}
