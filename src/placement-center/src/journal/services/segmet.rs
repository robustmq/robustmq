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

use grpc_clients::pool::ClientPool;
use metadata_struct::journal::node_extend::JournalNodeExtend;
use metadata_struct::journal::segment::{JournalSegment, Replica, SegmentConfig, SegmentStatus};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, DeleteSegmentReply, DeleteSegmentRequest,
};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rocksdb_engine::RocksDBEngine;

use super::shard::update_last_segment_by_shard;
use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::journal::cache::{load_journal_cache, JournalCacheManager};
use crate::journal::controller::call_node::{
    update_cache_by_set_segment, update_cache_by_set_segment_meta, update_cache_by_set_shard,
    JournalInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};

pub async fn create_segment_by_req(
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateNextSegmentRequest,
) -> Result<CreateNextSegmentReply, PlacementCenterError> {
    let mut shard = if let Some(shard) =
        engine_cache.get_shard(&req.cluster_name, &req.namespace, &req.shard_name)
    {
        shard
    } else {
        return Err(PlacementCenterError::ShardDoesNotExist(
            req.cluster_name.to_string(),
        ));
    };
    let next_segment_no = if let Some(segment_no) =
        engine_cache.next_segment_seq(&req.cluster_name, &req.namespace, &req.shard_name)
    {
        segment_no
    } else {
        return Err(PlacementCenterError::ShardDoesNotExist(shard.name()));
    };

    let mut shard_notice = false;
    // If the next Segment hasn't already been created, it triggers the creation of the next Segment
    if engine_cache
        .get_segment(
            &req.cluster_name,
            &req.namespace,
            &req.shard_name,
            next_segment_no,
        )
        .is_none()
    {
        let segment = build_next_segment(&shard, engine_cache, cluster_cache).await?;
        sync_save_segment_info(raft_machine_apply, &segment).await?;

        let metadata = JournalSegmentMetadata {
            cluster_name: segment.cluster_name.clone(),
            namespace: segment.namespace.clone(),
            shard_name: segment.shard_name.clone(),
            segment_seq: segment.segment_seq,
            start_offset: -1,
            end_offset: -1,
            start_timestamp: -1,
            end_timestamp: -1,
        };
        sync_save_segment_metadata_info(raft_machine_apply, &metadata).await?;
        update_cache_by_set_segment_meta(&req.cluster_name, call_manager, client_pool, metadata)
            .await?;
        update_last_segment_by_shard(
            raft_machine_apply,
            engine_cache,
            &mut shard,
            segment.segment_seq,
        )
        .await?;
        update_cache_by_set_segment(
            &req.cluster_name,
            call_manager,
            client_pool,
            segment.clone(),
        )
        .await?;
        shard_notice = true;
    }

    let active_segment = if let Some(segment) = engine_cache.get_segment(
        &req.cluster_name,
        &req.namespace,
        &req.shard_name,
        shard.active_segment_seq,
    ) {
        segment
    } else {
        load_journal_cache(engine_cache, rocksdb_engine_handler)?;
        return Err(PlacementCenterError::SegmentDoesNotExist(shard.name()));
    };

    // try fixed segment status
    if active_segment.status == SegmentStatus::SealUp
        || active_segment.status == SegmentStatus::PreDelete
        || active_segment.status == SegmentStatus::Deleteing
    {
        shard.active_segment_seq = next_segment_no;
        shard_notice = true;
    }
    if shard_notice {
        update_cache_by_set_shard(&req.cluster_name, call_manager, client_pool, shard).await?;
    }

    Ok(CreateNextSegmentReply {})
}

pub async fn delete_segment_by_req(
    engine_cache: &Arc<JournalCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSegmentRequest,
) -> Result<DeleteSegmentReply, PlacementCenterError> {
    if engine_cache
        .get_shard(&req.cluster_name, &req.namespace, &req.shard_name)
        .is_none()
    {
        return Err(PlacementCenterError::ShardDoesNotExist(
            req.cluster_name.to_string(),
        ));
    };

    let segment = if let Some(segment) = engine_cache.get_segment(
        &req.cluster_name,
        &req.namespace,
        &req.shard_name,
        req.segment_seq,
    ) {
        segment
    } else {
        return Err(PlacementCenterError::SegmentDoesNotExist(format!(
            "{}-{}",
            req.shard_name, req.segment_seq
        )));
    };

    update_segment_status(
        engine_cache,
        raft_machine_apply,
        &segment,
        SegmentStatus::PreDelete,
    )
    .await?;

    engine_cache.add_wait_delete_segment(&segment);

    update_cache_by_set_segment(
        &segment.cluster_name,
        call_manager,
        client_pool,
        segment.clone(),
    )
    .await?;

    Ok(DeleteSegmentReply::default())
}

pub(crate) async fn build_first_segment(
    shard_info: &JournalShard,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
) -> Result<JournalSegment, PlacementCenterError> {
    build_segment(shard_info, engine_cache, cluster_cache, 0).await
}

async fn build_next_segment(
    shard_info: &JournalShard,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
) -> Result<JournalSegment, PlacementCenterError> {
    let segment_no = if let Some(segment_no) = engine_cache.next_segment_seq(
        &shard_info.cluster_name,
        &shard_info.namespace,
        &shard_info.shard_name,
    ) {
        segment_no
    } else {
        return Err(PlacementCenterError::ShardDoesNotExist(
            shard_info.shard_name.clone(),
        ));
    };

    build_segment(shard_info, engine_cache, cluster_cache, segment_no).await
}

async fn build_segment(
    shard_info: &JournalShard,
    engine_cache: &Arc<JournalCacheManager>,
    cluster_cache: &Arc<PlacementCacheManager>,
    segment_no: u32,
) -> Result<JournalSegment, PlacementCenterError> {
    if let Some(segment) = engine_cache.get_segment(
        &shard_info.cluster_name,
        &shard_info.namespace,
        &shard_info.shard_name,
        segment_no,
    ) {
        return Ok(segment.clone());
    }

    let node_list = cluster_cache.get_broker_node_id_by_cluster(&shard_info.cluster_name);
    if node_list.len() < shard_info.replica as usize {
        return Err(PlacementCenterError::NotEnoughNodes(
            shard_info.replica,
            node_list.len() as u32,
        ));
    }

    //todo Get the node copies at random
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

    Ok(JournalSegment {
        cluster_name: shard_info.cluster_name.clone(),
        namespace: shard_info.namespace.clone(),
        shard_name: shard_info.shard_name.clone(),
        leader_epoch: 0,
        status: SegmentStatus::Idle,
        segment_seq: segment_no,
        leader: calc_leader_node(&replicas),
        replicas: replicas.clone(),
        isr: replicas.clone(),
        config: SegmentConfig {
            max_segment_size: 1024 * 1024 * 1024,
        },
    })
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

    //todo
    let data = serde_json::from_str::<JournalNodeExtend>(&node.extend)?;
    let fold_list = data.data_fold;
    let mut rng = thread_rng();
    let index = rng.gen_range(0..fold_list.len());
    let random_element = fold_list.get(index).unwrap();
    Ok(random_element.clone())
}

pub async fn update_segment_status(
    engine_cache: &Arc<JournalCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    segment: &JournalSegment,
    status: SegmentStatus,
) -> Result<(), PlacementCenterError> {
    let mut new_segment = segment.clone();
    new_segment.status = status;
    sync_save_segment_info(raft_machine_apply, &new_segment).await?;

    engine_cache.set_segment(&new_segment);

    Ok(())
}

pub async fn sync_save_segment_info(
    raft_machine_apply: &Arc<RaftMachineApply>,
    segment: &JournalSegment,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::JournalSetSegment,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_info(
    raft_machine_apply: &Arc<RaftMachineApply>,
    segment: &JournalSegment,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::JournalDeleteSegment,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

pub async fn sync_save_segment_metadata_info(
    raft_machine_apply: &Arc<RaftMachineApply>,
    segment: &JournalSegmentMetadata,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::JournalSetSegmentMetadata,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_metadata_info(
    raft_machine_apply: &Arc<RaftMachineApply>,
    segment: &JournalSegmentMetadata,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::JournalDeleteSegmentMetadata,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::tools::now_mills;
    use metadata_struct::journal::node_extend::JournalNodeExtend;
    use metadata_struct::placement::node::BrokerNode;
    use protocol::placement_center::placement_center_inner::ClusterType;
    use rocksdb_engine::RocksDBEngine;

    use super::calc_node_fold;
    use crate::core::cache::PlacementCacheManager;
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
            tcp_addr: "127.0.0.1:3110".to_string(),
            tcps_addr: "127.0.0.1:3110".to_string(),
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

    // #[tokio::test]
    // async fn create_segment_test() {
    //     let config = placement_center_test_conf();
    //     let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
    //         &storage_data_fold(&config.rocksdb.data_path),
    //         config.rocksdb.max_open_files.unwrap(),
    //         column_family_list(),
    //     ));
    //     let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
    //     let engine_cache = Arc::new(JournalCacheManager::new());
    //     let shard_info = JournalShard {
    //         shard_uid: unique_id(),
    //         cluster_name: config.cluster_name.clone(),
    //         namespace: "n1".to_string(),
    //         shard_name: "s1".to_string(),
    //         replica: 2,
    //         start_segment_seq: 0,
    //         active_segment_seq: 0,
    //         last_segment_seq: 0,
    //         create_time: now_mills(),
    //     };
    //     let segment_no = 1;
    //     let segment = create_segment(
    //         &shard_info,
    //         &engine_cache,
    //         &cluster_cache,
    //         &rocksdb_engine_handler,
    //         segment_no,
    //     );
    //     assert!(segment.is_err());

    //     let extend_info = JournalNodeExtend {
    //         data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
    //         tcp_addr: "127.0.0.1:3110".to_string(),
    //         tcps_addr: "127.0.0.1:3110".to_string(),
    //     };

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 1,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let segment = create_segment(
    //         &shard_info,
    //         &engine_cache,
    //         &cluster_cache,
    //         &rocksdb_engine_handler,
    //         segment_no,
    //     );
    //     assert!(segment.is_err());

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 2,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 3,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let segment = create_segment(
    //         &shard_info,
    //         &engine_cache,
    //         &cluster_cache,
    //         &rocksdb_engine_handler,
    //         segment_no,
    //     )
    //     .unwrap();
    //     assert_eq!(segment.cluster_name, config.cluster_name);
    //     assert_eq!(segment.namespace, shard_info.namespace);
    //     assert_eq!(segment.shard_name, shard_info.shard_name);
    //     assert_eq!(segment.segment_seq, segment_no);
    //     assert_eq!(segment.replicas.len(), 2);
    //     assert_eq!(segment.status, SegmentStatus::Idle);
    // }

    // #[tokio::test]
    // async fn create_first_segment_test() {
    //     let config = placement_center_test_conf();
    //     let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
    //         &storage_data_fold(&config.rocksdb.data_path),
    //         config.rocksdb.max_open_files.unwrap(),
    //         column_family_list(),
    //     ));
    //     let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
    //     let engine_cache = Arc::new(JournalCacheManager::new());
    //     let shard_info = JournalShard {
    //         shard_uid: unique_id(),
    //         cluster_name: config.cluster_name.clone(),
    //         namespace: "n1".to_string(),
    //         shard_name: "s1".to_string(),
    //         replica: 2,
    //         start_segment_seq: 0,
    //         active_segment_seq: 0,
    //         last_segment_seq: 0,
    //         create_time: now_mills(),
    //     };

    //     let extend_info = JournalNodeExtend {
    //         data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
    //         tcp_addr: "127.0.0.1:3110".to_string(),
    //         tcps_addr: "127.0.0.1:3110".to_string(),
    //     };

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 1,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 2,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 3,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let segment = create_first_segment(
    //         &shard_info,
    //         &engine_cache,
    //         &cluster_cache,
    //         &rocksdb_engine_handler,
    //     )
    //     .unwrap();

    //     assert_eq!(segment.cluster_name, config.cluster_name);
    //     assert_eq!(segment.namespace, shard_info.namespace);
    //     assert_eq!(segment.shard_name, shard_info.shard_name);
    //     assert_eq!(segment.segment_seq, 0);
    //     assert_eq!(segment.replicas.len(), 2);
    //     assert_eq!(segment.status, SegmentStatus::Idle);
    // }

    // #[tokio::test]
    // async fn create_next_segment_test() {
    //     let config = placement_center_test_conf();
    //     let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
    //         &storage_data_fold(&config.rocksdb.data_path),
    //         config.rocksdb.max_open_files.unwrap(),
    //         column_family_list(),
    //     ));
    //     let cluster_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
    //     let engine_cache = Arc::new(JournalCacheManager::new());
    //     let shard_info = JournalShard {
    //         shard_uid: unique_id(),
    //         cluster_name: config.cluster_name.clone(),
    //         namespace: "n1".to_string(),
    //         shard_name: "s1".to_string(),
    //         replica: 2,
    //         start_segment_seq: 0,
    //         active_segment_seq: 0,
    //         last_segment_seq: 0,
    //         create_time: now_mills(),
    //     };

    //     engine_cache.add_shard(&shard_info);

    //     let extend_info = JournalNodeExtend {
    //         data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
    //         tcp_addr: "127.0.0.1:3110".to_string(),
    //         tcps_addr: "127.0.0.1:3110".to_string(),
    //     };

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 1,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 2,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let node = BrokerNode {
    //         cluster_name: config.cluster_name.clone(),
    //         cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
    //         create_time: now_mills(),
    //         extend: serde_json::to_string(&extend_info).unwrap(),
    //         node_id: 3,
    //         node_inner_addr: "".to_string(),
    //         node_ip: "".to_string(),
    //     };
    //     cluster_cache.add_broker_node(node);

    //     let segment = create_next_segment(
    //         &shard_info,
    //         &engine_cache,
    //         &cluster_cache,
    //         &rocksdb_engine_handler,
    //     )
    //     .unwrap();

    //     assert_eq!(segment.cluster_name, config.cluster_name);
    //     assert_eq!(segment.namespace, shard_info.namespace);
    //     assert_eq!(segment.shard_name, shard_info.shard_name);
    //     assert_eq!(segment.segment_seq, shard_info.last_segment_seq + 1);
    //     assert_eq!(segment.replicas.len(), 2);
    //     assert_eq!(segment.status, SegmentStatus::Idle);
    // }
}
