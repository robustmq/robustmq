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
use metadata_struct::journal::segment::{JournalSegment, Replica, SegmentStatus};
use metadata_struct::journal::shard::JournalShard;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rocksdb_engine::RocksDBEngine;

use crate::cache::journal::JournalCacheManager;
use crate::cache::placement::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::storage::journal::segment::SegmentStorage;

pub fn create_first_segment(
    shard_info: &JournalShard,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<JournalSegment, PlacementCenterError> {
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
    shard_info: &JournalShard,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<JournalSegment, PlacementCenterError> {
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
    shard_info: &JournalShard,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_no: u32,
) -> Result<JournalSegment, PlacementCenterError> {
    // build segement
    let node_list = cluster_cache.get_broker_node_id_by_cluster(&shard_info.cluster_name);
    if node_list.len() < shard_info.replica as usize {
        return Err(PlacementCenterError::NotEnoughNodes(
            shard_info.replica,
            node_list.len() as u32,
        ));
    }

    // Get the node copies at random
    let mut rng = thread_rng();
    let node_ids: Vec<u64> = node_list
        .choose_multiple(&mut rng, shard_info.replica as usize)
        .cloned()
        .collect();
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

    if replicas.len() != (shard_info.replica as usize) {
        return Err(PlacementCenterError::NumberOfReplicasIsIncorrect(
            shard_info.replica,
            replicas.len(),
        ));
    }

    let segment = JournalSegment {
        cluster_name: shard_info.cluster_name.clone(),
        namespace: shard_info.namespace.clone(),
        shard_name: shard_info.shard_name.clone(),
        leader_epoch: 0,
        status: SegmentStatus::Idle,
        segment_seq: segment_no,
        leader: calc_leader_node(&replicas),
        replicas: replicas.clone(),
        isr: replicas.clone(),
    };

    // save segment
    let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
    segment_storage.save(segment.clone())?;

    // update cache
    engine_cache.add_segment(segment.clone());

    Ok(segment)
}

fn calc_leader_node(replicas: &[Replica]) -> u64 {
    replicas.first().unwrap().node_id
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
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::tools::{now_mills, unique_id};
    use metadata_struct::journal::node_extend::JournalNodeExtend;
    use metadata_struct::journal::segment::SegmentStatus;
    use metadata_struct::journal::shard::JournalShard;
    use metadata_struct::placement::node::BrokerNode;
    use protocol::placement_center::placement_center_inner::ClusterType;
    use rocksdb_engine::RocksDBEngine;

    use super::{calc_node_fold, create_first_segment, create_next_segment, create_segment};
    use crate::cache::journal::JournalCacheManager;
    use crate::cache::placement::PlacementCacheManager;
    use crate::storage::rocksdb::{column_family_list, storage_data_fold};

    #[tokio::test]
    async fn calc_node_fold_test() {
        let config = placement_center_test_conf();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler));
        let extend_info = JournalNodeExtend {
            data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
        };

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 1,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);
        let res = calc_node_fold(&cluster_cache, &config.cluster_name, 1).unwrap();
        println!("{}", res);
        assert!(!res.is_empty())
    }

    #[tokio::test]
    async fn create_segment_test() {
        let config = placement_center_test_conf();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let engine_cache = Arc::new(JournalCacheManager::new());
        let shard_info = JournalShard {
            shard_uid: unique_id(),
            cluster_name: config.cluster_name.clone(),
            namespace: "n1".to_string(),
            shard_name: "s1".to_string(),
            replica: 2,
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            create_time: now_mills(),
        };
        let segment_no = 1;
        let segment = create_segment(
            &shard_info,
            &engine_cache,
            &cluster_cache,
            &rocksdb_engine_handler,
            segment_no,
        );
        assert!(segment.is_err());

        let extend_info = JournalNodeExtend {
            data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
        };

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 1,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let segment = create_segment(
            &shard_info,
            &engine_cache,
            &cluster_cache,
            &rocksdb_engine_handler,
            segment_no,
        );
        assert!(segment.is_err());

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 2,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 3,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let segment = create_segment(
            &shard_info,
            &engine_cache,
            &cluster_cache,
            &rocksdb_engine_handler,
            segment_no,
        )
        .unwrap();
        assert_eq!(segment.cluster_name, config.cluster_name);
        assert_eq!(segment.namespace, shard_info.namespace);
        assert_eq!(segment.shard_name, shard_info.shard_name);
        assert_eq!(segment.segment_seq, segment_no);
        assert_eq!(segment.replicas.len(), 2);
        assert_eq!(segment.status, SegmentStatus::Idle);
    }

    #[tokio::test]
    async fn create_first_segment_test() {
        let config = placement_center_test_conf();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let engine_cache = Arc::new(JournalCacheManager::new());
        let shard_info = JournalShard {
            shard_uid: unique_id(),
            cluster_name: config.cluster_name.clone(),
            namespace: "n1".to_string(),
            shard_name: "s1".to_string(),
            replica: 2,
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            create_time: now_mills(),
        };

        let extend_info = JournalNodeExtend {
            data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
        };

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 1,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 2,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 3,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let segment = create_first_segment(
            &shard_info,
            &engine_cache,
            &cluster_cache,
            &rocksdb_engine_handler,
        )
        .unwrap();

        assert_eq!(segment.cluster_name, config.cluster_name);
        assert_eq!(segment.namespace, shard_info.namespace);
        assert_eq!(segment.shard_name, shard_info.shard_name);
        assert_eq!(segment.segment_seq, 0);
        assert_eq!(segment.replicas.len(), 2);
        assert_eq!(segment.status, SegmentStatus::Idle);
    }

    #[tokio::test]
    async fn create_next_segment_test() {
        let config = placement_center_test_conf();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let engine_cache = Arc::new(JournalCacheManager::new());
        let shard_info = JournalShard {
            shard_uid: unique_id(),
            cluster_name: config.cluster_name.clone(),
            namespace: "n1".to_string(),
            shard_name: "s1".to_string(),
            replica: 2,
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            create_time: now_mills(),
        };

        engine_cache.add_shard(&shard_info);

        let extend_info = JournalNodeExtend {
            data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
        };

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 1,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 2,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
            create_time: now_mills(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 3,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cluster_cache.add_broker_node(node);

        let segment = create_next_segment(
            &shard_info,
            &engine_cache,
            &cluster_cache,
            &rocksdb_engine_handler,
        )
        .unwrap();

        assert_eq!(segment.cluster_name, config.cluster_name);
        assert_eq!(segment.namespace, shard_info.namespace);
        assert_eq!(segment.shard_name, shard_info.shard_name);
        assert_eq!(segment.segment_seq, shard_info.last_segment_seq + 1);
        assert_eq!(segment.replicas.len(), 2);
        assert_eq!(segment.status, SegmentStatus::Idle);
    }
}
