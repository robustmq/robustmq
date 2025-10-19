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

use super::shard::update_last_segment_by_shard;
use crate::controller::journal::call_node::{
    update_cache_by_set_segment, update_cache_by_set_segment_meta, update_cache_by_set_shard,
    JournalInnerCallManager,
};
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::route::apply::StorageDriver;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::node_extend::JournalNodeExtend;
use metadata_struct::journal::segment::{
    str_to_segment_status, JournalSegment, Replica, SegmentConfig, SegmentStatus,
};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, DeleteSegmentReply, DeleteSegmentRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    UpdateSegmentMetaReply, UpdateSegmentMetaRequest, UpdateSegmentStatusReply,
    UpdateSegmentStatusRequest,
};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

pub async fn list_segment_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSegmentRequest,
) -> Result<ListSegmentReply, MetaServiceError> {
    let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
    let binary_segments =
        if req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            segment_storage.list_by_cluster(&req.cluster_name)?
        } else if !req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            segment_storage.list_by_namespace(&req.cluster_name, &req.namespace)?
        } else if !req.namespace.is_empty() && !req.shard_name.is_empty() && req.segment_no == -1 {
            segment_storage.list_by_shard(&req.cluster_name, &req.namespace, &req.shard_name)?
        } else {
            match segment_storage.get(
                &req.cluster_name,
                &req.namespace,
                &req.shard_name,
                req.segment_no as u32,
            )? {
                Some(segment) => vec![segment],
                None => Vec::new(),
            }
        };

    let segments_data = serde_json::to_vec(&binary_segments)?;

    Ok(ListSegmentReply {
        segments: segments_data,
    })
}

pub async fn create_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateNextSegmentRequest,
) -> Result<CreateNextSegmentReply, MetaServiceError> {
    let mut shard = if let Some(shard) =
        cache_manager.get_shard(&req.cluster_name, &req.namespace, &req.shard_name)
    {
        shard
    } else {
        return Err(MetaServiceError::ShardDoesNotExist(
            req.cluster_name.to_string(),
        ));
    };

    let next_segment_no = if let Some(segment_no) =
        cache_manager.next_segment_seq(&req.cluster_name, &req.namespace, &req.shard_name)
    {
        segment_no
    } else {
        return Err(MetaServiceError::ShardDoesNotExist(shard.name()));
    };

    if cache_manager.shard_idle_segment_num(&req.cluster_name, &req.namespace, &req.shard_name) >= 1
    {
        return Ok(CreateNextSegmentReply {});
    }

    let mut shard_notice = false;
    // If the next Segment hasn't already been created, it triggers the creation of the next Segment
    if cache_manager
        .get_segment(
            &req.cluster_name,
            &req.namespace,
            &req.shard_name,
            next_segment_no,
        )
        .is_none()
    {
        let segment = build_segment(&shard, cache_manager, next_segment_no).await?;
        sync_save_segment_info(raft_machine_apply, &segment).await?;

        let metadata = JournalSegmentMetadata {
            cluster_name: segment.cluster_name.clone(),
            namespace: segment.namespace.clone(),
            shard_name: segment.shard_name.clone(),
            segment_seq: segment.segment_seq,
            start_offset: if segment.segment_seq == 0 { 0 } else { -1 },
            end_offset: -1,
            start_timestamp: -1,
            end_timestamp: -1,
        };
        sync_save_segment_metadata_info(raft_machine_apply, &metadata).await?;

        update_last_segment_by_shard(
            raft_machine_apply,
            cache_manager,
            &mut shard,
            next_segment_no,
        )
        .await?;

        update_cache_by_set_segment_meta(&req.cluster_name, call_manager, client_pool, metadata)
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

    let active_segment = if let Some(segment) = cache_manager.get_segment(
        &req.cluster_name,
        &req.namespace,
        &req.shard_name,
        shard.active_segment_seq,
    ) {
        segment
    } else {
        return Err(MetaServiceError::SegmentDoesNotExist(shard.name()));
    };

    // try fixed segment status
    if active_segment.status == SegmentStatus::SealUp
        || active_segment.status == SegmentStatus::PreDelete
        || active_segment.status == SegmentStatus::Deleting
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
    cache_manager: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSegmentRequest,
) -> Result<DeleteSegmentReply, MetaServiceError> {
    if cache_manager
        .get_shard(&req.cluster_name, &req.namespace, &req.shard_name)
        .is_none()
    {
        return Err(MetaServiceError::ShardDoesNotExist(
            req.cluster_name.to_string(),
        ));
    };

    let mut segment = if let Some(segment) = cache_manager.get_segment(
        &req.cluster_name,
        &req.namespace,
        &req.shard_name,
        req.segment_seq,
    ) {
        segment
    } else {
        return Err(MetaServiceError::SegmentDoesNotExist(format!(
            "{}-{}",
            req.shard_name, req.segment_seq
        )));
    };

    if segment.status != SegmentStatus::SealUp {
        return Err(MetaServiceError::NoAllowDeleteSegment(
            segment.name(),
            segment.status.to_string(),
        ));
    }

    update_segment_status(
        cache_manager,
        raft_machine_apply,
        &segment,
        SegmentStatus::PreDelete,
    )
    .await?;

    segment.status = SegmentStatus::PreDelete;
    cache_manager.add_wait_delete_segment(&segment);

    update_cache_by_set_segment(
        &segment.cluster_name,
        call_manager,
        client_pool,
        segment.clone(),
    )
    .await?;

    Ok(DeleteSegmentReply::default())
}

pub async fn update_segment_status_req(
    cache_manager: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSegmentStatusRequest,
) -> Result<UpdateSegmentStatusReply, MetaServiceError> {
    let mut segment = if let Some(segment) = cache_manager.get_segment(
        &req.cluster_name,
        &req.namespace,
        &req.shard_name,
        req.segment_seq,
    ) {
        segment
    } else {
        return Err(MetaServiceError::SegmentDoesNotExist(format!(
            "{}_{}",
            req.shard_name, req.segment_seq
        )));
    };

    if segment.status.to_string() != req.cur_status {
        return Err(MetaServiceError::SegmentStateError(
            segment.name(),
            segment.status.to_string(),
            req.cur_status.clone(),
        ));
    }

    let new_status = str_to_segment_status(&req.next_status)?;
    segment.status = new_status;

    sync_save_segment_info(raft_machine_apply, &segment).await?;
    update_cache_by_set_segment(
        &req.cluster_name,
        call_manager,
        client_pool,
        segment.clone(),
    )
    .await?;
    Ok(UpdateSegmentStatusReply::default())
}

pub async fn list_segment_meta_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSegmentMetaRequest,
) -> Result<ListSegmentMetaReply, MetaServiceError> {
    let storage = SegmentMetadataStorage::new(rocksdb_engine_handler.clone());
    let binary_segments =
        if req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            storage.list_by_cluster(&req.cluster_name)?
        } else if !req.namespace.is_empty() && req.shard_name.is_empty() && req.segment_no == -1 {
            storage.list_by_namespace(&req.cluster_name, &req.namespace)?
        } else if !req.namespace.is_empty() && !req.shard_name.is_empty() && req.segment_no == -1 {
            storage.list_by_shard(&req.cluster_name, &req.namespace, &req.shard_name)?
        } else {
            match storage.get(
                &req.cluster_name,
                &req.namespace,
                &req.shard_name,
                req.segment_no as u32,
            )? {
                Some(segment_meta) => vec![segment_meta],
                None => Vec::new(),
            }
        };

    let segments_data = serde_json::to_vec(&binary_segments)?;

    Ok(ListSegmentMetaReply {
        segments: segments_data,
    })
}

pub async fn update_segment_meta_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<JournalInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSegmentMetaRequest,
) -> Result<UpdateSegmentMetaReply, MetaServiceError> {
    if req.cluster_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            req.cluster_name.clone(),
        ));
    }

    if cache_manager
        .get_segment(
            &req.cluster_name,
            &req.namespace,
            &req.shard_name,
            req.segment_no,
        )
        .is_none()
    {
        return Err(MetaServiceError::SegmentDoesNotExist(format!(
            "{}_{}",
            req.shard_name, req.segment_no
        )));
    };

    let mut segment_meta = if let Some(meta) = cache_manager.get_segment_meta(
        &req.cluster_name,
        &req.namespace,
        &req.shard_name,
        req.segment_no,
    ) {
        meta
    } else {
        return Err(MetaServiceError::SegmentMetaDoesNotExist(format!(
            "{}_{}",
            req.shard_name, req.segment_no
        )));
    };

    if req.start_offset > 0 {
        segment_meta.start_offset = req.start_offset;
    }

    if req.end_offset > 0 {
        segment_meta.end_offset = req.end_offset;
    }

    if req.start_timestamp > 0 {
        segment_meta.start_timestamp = req.start_timestamp;
    }

    if req.end_timestamp > 0 {
        segment_meta.end_timestamp = req.end_timestamp;
    }

    sync_save_segment_metadata_info(raft_machine_apply, &segment_meta).await?;

    update_cache_by_set_segment_meta(&req.cluster_name, call_manager, client_pool, segment_meta)
        .await?;

    Ok(UpdateSegmentMetaReply::default())
}

pub async fn build_segment(
    shard_info: &JournalShard,
    cache_manager: &Arc<CacheManager>,
    segment_no: u32,
) -> Result<JournalSegment, MetaServiceError> {
    if let Some(segment) = cache_manager.get_segment(
        &shard_info.cluster_name,
        &shard_info.namespace,
        &shard_info.shard_name,
        segment_no,
    ) {
        return Ok(segment.clone());
    }

    let node_list = cache_manager.get_broker_node_id_by_cluster(&shard_info.cluster_name);
    if node_list.len() < shard_info.config.replica_num as usize {
        return Err(MetaServiceError::NotEnoughNodes(
            shard_info.config.replica_num,
            node_list.len() as u32,
        ));
    }

    //todo Get the node copies at random
    let mut rng = thread_rng();
    let node_ids: Vec<u64> = node_list
        .choose_multiple(&mut rng, shard_info.config.replica_num as usize)
        .cloned()
        .collect();
    let mut replicas = Vec::new();
    for i in 0..node_ids.len() {
        let node_id = *node_ids.get(i).unwrap();
        let fold = calc_node_fold(cache_manager, &shard_info.cluster_name, node_id)?;
        replicas.push(Replica {
            replica_seq: i as u64,
            node_id,
            fold,
        });
    }

    if replicas.len() != (shard_info.config.replica_num as usize) {
        return Err(MetaServiceError::NumberOfReplicasIsIncorrect(
            shard_info.config.replica_num,
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
        isr: replicas.iter().map(|rep| rep.node_id).collect(),
        config: SegmentConfig {
            max_segment_size: shard_info.config.max_segment_size,
        },
    })
}

fn calc_leader_node(replicas: &[Replica]) -> u64 {
    replicas.first().unwrap().node_id
}

fn calc_node_fold(
    cache_manager: &Arc<CacheManager>,
    cluster_name: &str,
    node_id: u64,
) -> Result<String, MetaServiceError> {
    let node = if let Some(node) = cache_manager.get_broker_node(cluster_name, node_id) {
        node
    } else {
        return Err(MetaServiceError::NodeDoesNotExist(node_id));
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
    cache_manager: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    segment: &JournalSegment,
    status: SegmentStatus,
) -> Result<(), MetaServiceError> {
    let mut new_segment = segment.clone();
    new_segment.status = status;
    sync_save_segment_info(raft_machine_apply, &new_segment).await?;

    cache_manager.set_segment(&new_segment);

    Ok(())
}

pub async fn sync_save_segment_info(
    raft_machine_apply: &Arc<StorageDriver>,
    segment: &JournalSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalSetSegment,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_info(
    raft_machine_apply: &Arc<StorageDriver>,
    segment: &JournalSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalDeleteSegment,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_save_segment_metadata_info(
    raft_machine_apply: &Arc<StorageDriver>,
    segment: &JournalSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalSetSegmentMetadata,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_metadata_info(
    raft_machine_apply: &Arc<StorageDriver>,
    segment: &JournalSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalDeleteSegmentMetadata,
        serde_json::to_vec(&segment)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

#[cfg(test)]
mod tests {
    use super::calc_node_fold;
    use crate::core::cache::CacheManager;
    use broker_core::rocksdb::{column_family_list, storage_data_fold};
    use common_base::tools::now_second;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::journal::node_extend::JournalNodeExtend;
    use metadata_struct::meta::node::BrokerNode;
    use rocksdb_engine::RocksDBEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn calc_node_fold_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let cache_manager = Arc::new(CacheManager::new(rocksdb_engine_handler));
        let extend_info = JournalNodeExtend {
            data_fold: vec!["/tmp/t1".to_string(), "/tmp/t2".to_string()],
            tcp_addr: "127.0.0.1:3110".to_string(),
        };

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            roles: Vec::new(),
            register_time: now_second(),
            start_time: now_second(),
            extend: serde_json::to_string(&extend_info).unwrap(),
            node_id: 1,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
        };
        cache_manager.add_broker_node(node);
        let res = calc_node_fold(&cache_manager, &config.cluster_name, 1).unwrap();
        println!("{res}");
        assert!(!res.is_empty())
    }

    // #[tokio::test]
    // async fn create_segment_test() {
    //     let config = broker_config();;
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
    //     let config = broker_config();;
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
    //     let config = broker_config();;
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
