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

use common_base::tools::{now_mills, unique_id};
use grpc_clients::poll::ClientPool;
use prost::Message as _;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentRequest, CreateShardRequest, DeleteSegmentRequest,
};

use crate::cache::journal::JournalCacheManager;
use crate::cache::placement::PlacementCacheManager;
use crate::controller::journal::call_node::{
    update_cache_by_add_segment, update_cache_by_add_shard, update_cache_by_delete_segment,
    update_cache_by_delete_shard,
};
use crate::core::error::PlacementCenterError;
use crate::core::journal::segmet::{create_first_segment, create_next_segment};
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::shard::{ShardInfo, ShardStorage};
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Clone)]
pub struct DataRouteJournal {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    engine_cache: Arc<JournalCacheManager>,
    cluster_cache: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
}

impl DataRouteJournal {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        engine_cache: Arc<JournalCacheManager>,
        cluster_cache: Arc<PlacementCacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        DataRouteJournal {
            rocksdb_engine_handler,
            engine_cache,
            cluster_cache,
            client_poll,
        }
    }
    pub fn create_shard(&self, value: Vec<u8>) -> Result<Vec<u8>, PlacementCenterError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())?;

        let shard_info = ShardInfo {
            shard_uid: unique_id(),
            cluster_name: req.cluster_name.clone(),
            namespace: req.namespace.clone(),
            shard_name: req.shard_name.clone(),
            replica: req.replica,
            storage_mode: req.storage_model,
            active_segment_seq: 0,
            last_segment_seq: 0,
            create_time: now_mills(),
        };

        // Save Shard && Update Cache
        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        shard_storage.save(&shard_info)?;
        self.engine_cache.add_shard(&shard_info);

        // Create your first Segment
        let segment = create_first_segment(
            &shard_info,
            &self.engine_cache,
            &self.cluster_cache,
            &self.rocksdb_engine_handler,
        )?;

        // Update storage engine node cache
        update_cache_by_add_shard(
            req.cluster_name.clone(),
            self.cluster_cache.clone(),
            self.client_poll.clone(),
            shard_info,
        );

        update_cache_by_add_segment(
            req.cluster_name.clone(),
            self.cluster_cache.clone(),
            self.client_poll.clone(),
            segment.clone(),
        );

        Ok(serde_json::to_vec(&segment)?)
    }

    pub fn delete_shard(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())?;
        let cluster_name = req.cluster_name;
        let namespace = req.namespace;
        let shard_name = req.shard_name;

        let res = self
            .engine_cache
            .get_shard(&cluster_name, &namespace, &shard_name);
        if res.is_none() {
            return Err(PlacementCenterError::ShardDoesNotExist(shard_name));
        }

        let shard = res.unwrap();

        // Delete all segment
        let segment_storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        let segment_list = segment_storage.list_by_shard(&cluster_name, &shard_name)?;
        for segment in segment_list {
            segment_storage.delete(&cluster_name, &shard_name, segment.segment_seq)?;
        }

        // Delete shard info
        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        shard_storage.delete(&cluster_name, &namespace, &shard_name)?;

        self.engine_cache
            .remove_shard(&cluster_name, &namespace, &shard_name);

        // Update storage engine node cache
        update_cache_by_delete_shard(
            cluster_name,
            self.cluster_cache.clone(),
            self.client_poll.clone(),
            shard,
        );

        Ok(())
    }

    pub fn create_next_segment(&self, value: Vec<u8>) -> Result<Vec<u8>, PlacementCenterError> {
        let req = CreateNextSegmentRequest::decode(value.as_ref())?;

        let cluster_name = req.cluster_name;
        let shard_name = req.shard_name;
        let namespace = req.namespace;

        let res = self
            .engine_cache
            .get_shard(&cluster_name, &namespace, &shard_name);
        if res.is_none() {
            return Err(PlacementCenterError::ShardDoesNotExist(shard_name));
        }

        let shard = res.unwrap();

        if (shard.last_segment_seq - shard.active_segment_seq) >= req.active_segment_next_num {
            return Err(PlacementCenterError::ShardHasEnoughSegment(shard_name));
        }

        let segment = create_next_segment(
            &shard,
            &self.engine_cache,
            &self.cluster_cache,
            &self.rocksdb_engine_handler,
        )?;

        update_cache_by_add_segment(
            cluster_name,
            self.cluster_cache.clone(),
            self.client_poll.clone(),
            segment.clone(),
        );

        Ok(serde_json::to_vec(&segment)?)
    }

    pub fn delete_segment(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req: DeleteSegmentRequest = DeleteSegmentRequest::decode(value.as_ref())?;
        let cluster_name = req.cluster_name;
        let namespace = req.namespace;
        let shard_name = req.shard_name;
        let segment_seq = req.segment_seq;

        let shard_res = self
            .engine_cache
            .get_shard(&cluster_name, &namespace, &shard_name);
        if shard_res.is_none() {
            return Err(PlacementCenterError::ShardDoesNotExist(shard_name));
        }

        let segment_res =
            self.engine_cache
                .get_segment(&cluster_name, &namespace, &shard_name, segment_seq);

        if segment_res.is_none() {
            return Err(PlacementCenterError::SegmentDoesNotExist(shard_name));
        }

        let segment = segment_res.unwrap();

        let segment_storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        segment_storage.delete(&cluster_name, &shard_name, segment_seq)?;
        self.engine_cache
            .remove_segment(&cluster_name, &namespace, &shard_name, segment_seq);

        update_cache_by_delete_segment(
            cluster_name,
            self.cluster_cache.clone(),
            self.client_poll.clone(),
            segment.clone(),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn create_shard_test() {
        // todo
    }

    #[tokio::test]
    async fn delete_shard_test() {
        // todo
    }

    #[tokio::test]
    async fn create_next_segment_test() {
        // todo
    }

    #[tokio::test]
    async fn delete_segment_test() {
        // todo
    }
}
