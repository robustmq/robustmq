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

use common_base::error::common::CommonError;
use common_base::tools::{now_mills, unique_id};
use prost::Message as _;
use protocol::placement_center::placement_center_journal::{
    CreateSegmentRequest, CreateShardRequest, DeleteSegmentRequest,
};

use crate::cache::journal::JournalCacheManager;
use crate::cache::placement::PlacementCacheManager;
use crate::controller::journal::segment_replica::SegmentReplicaAlgorithm;
use crate::storage::journal::segment::{SegmentInfo, SegmentStatus, SegmentStorage};
use crate::storage::journal::shard::{ShardInfo, ShardStorage};
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Debug, Clone)]
pub struct DataRouteJournal {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    engine_cache: Arc<JournalCacheManager>,
    cluster_cache: Arc<PlacementCacheManager>,
}

impl DataRouteJournal {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        engine_cache: Arc<JournalCacheManager>,
        cluster_cache: Arc<PlacementCacheManager>,
    ) -> Self {
        DataRouteJournal {
            rocksdb_engine_handler,
            engine_cache,
            cluster_cache,
        }
    }
    pub fn create_shard(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())?;

        let shard_info = ShardInfo {
            shard_uid: unique_id(),
            cluster_name: req.cluster_name.clone(),
            shard_name: req.shard_name.clone(),
            replica: req.replica,
            last_segment_seq: 0,
            create_time: now_mills(),
        };

        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        shard_storage.save(shard_info.clone())?;

        // upate cache
        self.engine_cache.add_shard(shard_info);

        // Create segments according to the built-in pre-created segment strategy.
        // Between 0 and N segments may be created.
        self.pre_create_segment()?;

        // todo maybe update storage engine node cache
        Ok(())
    }

    pub fn delete_shard(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req: CreateShardRequest = CreateShardRequest::decode(value.as_ref())?;
        let cluster_name = req.cluster_name.clone();
        let shard_name = req.shard_name.clone();
        // todo delete all segment

        // delete shard info
        let shard_storage = ShardStorage::new(self.rocksdb_engine_handler.clone());
        shard_storage.delete(&cluster_name, &shard_name)?;

        self.engine_cache
            .remove_shard(cluster_name.clone(), shard_name.clone());

        Ok(())
    }

    pub fn create_segment(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req: CreateSegmentRequest = CreateSegmentRequest::decode(value.as_ref())?;

        let cluster_name = req.cluster_name;
        let shard_name = req.shard_name;

        let segment_seq = self
            .engine_cache
            .next_segment_seq(&cluster_name, &shard_name);

        let repcli_algo =
            SegmentReplicaAlgorithm::new(self.cluster_cache.clone(), self.engine_cache.clone());
        let segment_info = SegmentInfo {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            replicas: repcli_algo.calc_replica_distribution(segment_seq),
            replica_leader: 0,
            segment_seq,
            status: SegmentStatus::Idle,
        };
        let segment_storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        segment_storage.save(segment_info.clone())?;
        self.engine_cache.add_segment(segment_info);
        Ok(())
    }

    pub fn delete_segment(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req: DeleteSegmentRequest = DeleteSegmentRequest::decode(value.as_ref())?;
        let cluster_name = req.cluster_name;
        let shard_name = req.shard_name;
        let segment_seq = req.segment_seq;

        let segment_storage = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        segment_storage.delete(&cluster_name, &shard_name, segment_seq)?;
        self.engine_cache
            .remove_segment(cluster_name.clone(), shard_name.clone(), segment_seq);
        Ok(())
    }

    pub fn pre_create_segment(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
