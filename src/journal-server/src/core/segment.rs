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

use std::fs::remove_dir_all;
use std::path::Path;
use std::sync::Arc;

use common_base::config::journal_server::journal_server_conf;
use log::error;
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileRequest, GetSegmentDeleteStatusRequest,
};

use super::cache::CacheManager;
use super::error::JournalServerError;
use crate::segment::file::{data_file_segment, open_segment_write};
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

pub fn delete_local_segment(
    cache_manager: Arc<CacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    req: DeleteSegmentFileRequest,
) -> Result<(), JournalServerError> {
    let segment_iden = SegmentIdentity {
        namespace: req.namespace,
        shard_name: req.shard_name,
        segment_seq: req.segment,
    };

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

        let segment_file = data_file_segment(&data_fold, req.segment);
        if Path::new(&segment_file).exists() {
            if let Err(e) = remove_dir_all(&segment_file) {
                error!("{}", e);
            }
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

    let segment = if let Some(segment) = cache_manager.get_segment(&segment_iden) {
        segment
    } else {
        return Ok(false);
    };

    // Does the file exist
    let (segment_write, _) = open_segment_write(cache_manager, &segment_iden).await?;

    Ok(!segment_write.exists())
}
