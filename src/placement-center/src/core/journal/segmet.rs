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

use metadata_struct::journal::node_extend::JournalNodeExtend;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rocksdb_engine::RocksDBEngine;

use crate::cache::journal::JournalCacheManager;
use crate::cache::placement::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::storage::journal::segment::{Replica, SegmentInfo, SegmentStatus, SegmentStorage};
use crate::storage::journal::shard::ShardInfo;

pub fn create_first_segment(
    shard_info: &ShardInfo,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<SegmentInfo, PlacementCenterError> {
    let segment_no = 0;
    if let Some(segment) = engine_cache.get_segment(
        &shard_info.cluster_name,
        &shard_info.namespace,
        &shard_info.shard_name,
        segment_no,
    ) {
        return Ok(segment.clone());
    }

    let segment = create_segment(
        shard_info,
        engine_cache,
        cluster_cache,
        rocksdb_engine_handler,
        segment_no,
    )?;

    Ok(segment)
}

pub fn create_next_segment(
    shard_info: &ShardInfo,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<SegmentInfo, PlacementCenterError> {
    let segment_no_res = engine_cache.next_segment_seq(
        &shard_info.cluster_name,
        &shard_info.namespace,
        &shard_info.shard_name,
    );
    if segment_no_res.is_none() {
        return Err(PlacementCenterError::ShardDoesNotExist(
            shard_info.shard_name.clone(),
        ));
    }

    let segment_no = segment_no_res.unwrap();
    if let Some(segment) = engine_cache.get_segment(
        &shard_info.cluster_name,
        &shard_info.namespace,
        &shard_info.shard_name,
        segment_no,
    ) {
        return Ok(segment.clone());
    }

    let segment = create_segment(
        shard_info,
        engine_cache,
        cluster_cache,
        rocksdb_engine_handler,
        segment_no,
    )?;

    Ok(segment)
}

fn create_segment(
    shard_info: &ShardInfo,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_no: u32,
) -> Result<SegmentInfo, PlacementCenterError> {
    // build segement
    let node_list = cluster_cache.get_broker_node_id_by_cluster(&shard_info.cluster_name);
    if node_list.len() < shard_info.replica as usize {
        return Err(PlacementCenterError::NotEnoughNodes(
            shard_info.replica,
            node_list.len() as u32,
        ));
    }

    let mut segment = SegmentInfo {
        cluster_name: shard_info.cluster_name.clone(),
        namespace: shard_info.namespace.clone(),
        shard_name: shard_info.shard_name.clone(),
        status: SegmentStatus::Idle,
        ..Default::default()
    };

    // Get the node copies at random
    let mut rng = thread_rng();
    let node_ids: Vec<u64> = node_list.choose_multiple(&mut rng, 3).cloned().collect();
    let mut replicas = Vec::new();
    for i in 0..node_ids.len() {
        let node_id = *node_ids.get(i).unwrap();
        let fold = calc_node_fold(cluster_cache, &shard_info.cluster_name, node_id)?;
        replicas.push(Replica {
            replica_seq: i as u64,
            node_id,
            fold,
        });
    }
    segment.segment_seq = segment_no;
    segment.replicas = replicas;

    // save segment
    let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
    segment_storage.save(segment.clone())?;

    // update cache
    engine_cache.add_segment(segment.clone());

    Ok(segment)
}

fn calc_node_fold(
    cluster_cache: &Arc<PlacementCacheManager>,
    cluster_name: &str,
    node_id: u64,
) -> Result<String, PlacementCenterError> {
    let node = if let Some(node) = cluster_cache.get_broker_node(cluster_name, node_id) {
        node
    } else {
        return Err(PlacementCenterError::NodeDoesNotExist(node_id));
    };

    let data = serde_json::from_str::<JournalNodeExtend>(&node.extend)?;
    let fold_list = data.data_fold;
    let mut rng = thread_rng();
    let index = rng.gen_range(0..fold_list.len());
    let random_element = fold_list.get(index).unwrap();
    Ok(random_element.clone())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn calc_node_fold_test() {
        // todo
    }

    #[tokio::test]
    async fn create_segment_test() {
        // todo
    }
}
