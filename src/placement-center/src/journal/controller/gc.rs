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

use crate::journal::cache::JournalCacheManager;

pub fn gc_shard_thread(engine_cache: &Arc<JournalCacheManager>) {

    // // Raft state machine is used to store Node data
    // let data = StorageData::new(
    //     StorageDataType::JournalDeleteShard,
    //     DeleteShardRequest::encode_to_vec(&req),
    // );
    // if let Some(data) = raft_machine_apply.client_write(data).await? {
    //     update_cache_by_delete_shard(&req.cluster_name, &call_manager, &client_pool, shard).await?;
    // } else {
    //     return Err(PlacementCenterError::ExecutionResultIsEmpty);
    // }

    // let segment_storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
    // let segment_list = segment_storage.list_by_shard(&cluster_name, &namespace, &shard_name)?;
    // for segment in segment_list {
    //     segment_storage.delete(&cluster_name, &namespace, &shard_name, segment.segment_seq)?;
    // }
}

pub fn gc_segment_thread(engine_cache: &Arc<JournalCacheManager>) {}
