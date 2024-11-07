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
use protocol::journal_server::journal_inner::{
    DeleteShardFileRequest, GetShardDeleteStatusRequest,
};

use super::cache::CacheManager;
use super::error::JournalServerError;
use crate::segment::fold::data_fold_shard;

pub fn delete_local_shard(
    cache_manager: Arc<CacheManager>,
    req: DeleteShardFileRequest,
) -> Result<(), JournalServerError> {
    let shard = if let Some(shard) = cache_manager.get_shard(&req.namespace, &req.shard_name) {
        shard
    } else {
        return Err(JournalServerError::ShardNotExist(
            req.shard_name.to_string(),
        ));
    };

    tokio::spawn(async move {
        // delete file
        let conf = journal_server_conf();
        for data_fold in conf.storage.data_path.iter() {
            let shard_fold_name = data_fold_shard(&req.namespace, &req.shard_name, data_fold);
            if Path::new(data_fold).exists() {
                match remove_dir_all(shard_fold_name) {
                    Ok(()) => {}
                    Err(e) => {}
                }
            }
        }

        // delete cache
        cache_manager.delete_shard(&req.namespace, &req.shard_name);
    });
    Ok(())
}

pub fn get_delete_shard_status(
    cache_manager: &Arc<CacheManager>,
    req: &GetShardDeleteStatusRequest,
) -> Result<bool, JournalServerError> {
    // Does the file exist
    let mut fold_exist = false;
    let conf = journal_server_conf();
    for data_fold in conf.storage.data_path.iter() {
        let shard_fold_name = data_fold_shard(&req.namespace, &req.shard_name, data_fold);
        fold_exist = Path::new(data_fold).exists();
    }

    // Does the cache exist
    let cache_exist = cache_manager.shard_is_exists(&req.namespace, &req.shard_name);

    Ok(!fold_exist && !cache_exist)
}
