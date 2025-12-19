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
use crate::controller::call_broker::call::BrokerCallManager;
use crate::controller::call_broker::storage::{
    update_cache_by_set_segment, update_cache_by_set_segment_meta, update_cache_by_set_shard,
};
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use bytes::Bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::storage::segment::{
    str_to_segment_status, JournalSegment, Replica, SegmentConfig, SegmentStatus,
};
use metadata_struct::storage::segment_meta::JournalSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, DeleteSegmentReply, DeleteSegmentRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    UpdateSegmentMetaReply, UpdateSegmentMetaRequest, UpdateSegmentStatusReply,
    UpdateSegmentStatusRequest,
};
use rand::{thread_rng, Rng};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn list_segment_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSegmentRequest,
) -> Result<ListSegmentReply, MetaServiceError> {
    let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
    let binary_segments = if req.shard_name.is_empty() && req.segment_no == -1 {
        segment_storage.all_segment()?
    } else if !req.shard_name.is_empty() && req.segment_no == -1 {
        segment_storage.list_by_shard(&req.shard_name)?
    } else {
        match segment_storage.get(&req.shard_name, req.segment_no as u32)? {
            Some(segment) => vec![segment],
            None => Vec::new(),
        }
    };

    let segments_data = binary_segments
        .into_iter()
        .map(|segment| segment.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListSegmentReply {
        segments: segments_data,
    })
}

pub async fn create_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateNextSegmentRequest,
) -> Result<CreateNextSegmentReply, MetaServiceError> {
    let mut shard = if let Some(shard) = cache_manager.get_shard(&req.shard_name) {
        shard
    } else {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    let next_segment_no = if let Some(segment_no) = cache_manager.next_segment_seq(&req.shard_name)
    {
        segment_no
    } else {
        return Err(MetaServiceError::ShardDoesNotExist(shard.shard_name));
    };

    if cache_manager.shard_idle_segment_num(&req.shard_name) >= 1 {
        return Ok(CreateNextSegmentReply {});
    }

    let mut shard_notice = false;
    // If the next Segment hasn't already been created, it triggers the creation of the next Segment
    if cache_manager
        .get_segment(&req.shard_name, next_segment_no)
        .is_none()
    {
        let segment = build_segment(&shard, cache_manager, next_segment_no).await?;
        sync_save_segment_info(raft_manager, &segment).await?;

        let metadata = JournalSegmentMetadata {
            shard_name: segment.shard_name.clone(),
            segment_seq: segment.segment_seq,
            start_offset: if segment.segment_seq == 0 { 0 } else { -1 },
            end_offset: -1,
            start_timestamp: -1,
            end_timestamp: -1,
        };
        sync_save_segment_metadata_info(raft_manager, &metadata).await?;

        update_last_segment_by_shard(raft_manager, cache_manager, &mut shard, next_segment_no)
            .await?;

        update_cache_by_set_segment_meta(call_manager, client_pool, metadata).await?;
        update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;

        shard_notice = true;
    }

    let active_segment = if let Some(segment) =
        cache_manager.get_segment(&req.shard_name, shard.active_segment_seq)
    {
        segment
    } else {
        return Err(MetaServiceError::SegmentDoesNotExist(shard.shard_name));
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
        update_cache_by_set_shard(call_manager, client_pool, shard).await?;
    }

    Ok(CreateNextSegmentReply {})
}

pub async fn delete_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSegmentRequest,
) -> Result<DeleteSegmentReply, MetaServiceError> {
    if cache_manager.get_shard(&req.shard_name).is_none() {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    let mut segment =
        if let Some(segment) = cache_manager.get_segment(&req.shard_name, req.segment_seq) {
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
        raft_manager,
        &segment,
        SegmentStatus::PreDelete,
    )
    .await?;

    segment.status = SegmentStatus::PreDelete;
    cache_manager.add_wait_delete_segment(&segment);

    update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;

    Ok(DeleteSegmentReply::default())
}

pub async fn update_segment_status_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSegmentStatusRequest,
) -> Result<UpdateSegmentStatusReply, MetaServiceError> {
    let mut segment =
        if let Some(segment) = cache_manager.get_segment(&req.shard_name, req.segment_seq) {
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

    sync_save_segment_info(raft_manager, &segment).await?;
    update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;
    Ok(UpdateSegmentStatusReply::default())
}

pub async fn list_segment_meta_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSegmentMetaRequest,
) -> Result<ListSegmentMetaReply, MetaServiceError> {
    let storage = SegmentMetadataStorage::new(rocksdb_engine_handler.clone());
    let binary_segments = if req.shard_name.is_empty() && req.segment_no == -1 {
        storage.all_segment()?
    } else if !req.shard_name.is_empty() && req.segment_no == -1 {
        storage.list_by_shard(&req.shard_name)?
    } else {
        match storage.get(&req.shard_name, req.segment_no as u32)? {
            Some(segment_meta) => vec![segment_meta],
            None => Vec::new(),
        }
    };

    let segments_data = binary_segments
        .into_iter()
        .map(|segment| segment.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListSegmentMetaReply {
        segments: segments_data,
    })
}

pub async fn update_segment_meta_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSegmentMetaRequest,
) -> Result<UpdateSegmentMetaReply, MetaServiceError> {
    if req.shard_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            " shard_name".to_string(),
        ));
    }

    if cache_manager
        .get_segment(&req.shard_name, req.segment_no)
        .is_none()
    {
        return Err(MetaServiceError::SegmentDoesNotExist(format!(
            "{}_{}",
            req.shard_name, req.segment_no
        )));
    };

    let mut segment_meta =
        if let Some(meta) = cache_manager.get_segment_meta(&req.shard_name, req.segment_no) {
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

    sync_save_segment_metadata_info(raft_manager, &segment_meta).await?;

    update_cache_by_set_segment_meta(call_manager, client_pool, segment_meta).await?;

    Ok(UpdateSegmentMetaReply::default())
}

pub async fn build_segment(
    shard_info: &EngineShard,
    cache_manager: &Arc<CacheManager>,
    segment_no: u32,
) -> Result<JournalSegment, MetaServiceError> {
    if let Some(segment) = cache_manager.get_segment(&shard_info.shard_name, segment_no) {
        return Ok(segment);
    }

    let node_list: Vec<BrokerNode> = cache_manager
        .node_list
        .iter()
        .map(|raw| raw.clone())
        .collect();

    if node_list.len() < shard_info.config.replica_num as usize {
        return Err(MetaServiceError::NotEnoughNodes(
            shard_info.config.replica_num,
            node_list.len() as u32,
        ));
    }

    //todo Get the node copies at random
    let node_ids: Vec<u64> = node_list.iter().map(|raw| raw.node_id).collect();
    println!("{:?}", node_ids);
    let mut replicas = Vec::new();
    for i in 0..node_ids.len() {
        let node_id = *node_ids.get(i).unwrap();
        let fold = calc_node_fold(cache_manager, node_id)?;
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
    node_id: u64,
) -> Result<String, MetaServiceError> {
    let node = if let Some(node) = cache_manager.get_broker_node(node_id) {
        node
    } else {
        return Err(MetaServiceError::NodeDoesNotExist(node_id));
    };

    let fold_list = node.storage_fold.clone();
    let mut rng = thread_rng();
    let index = rng.gen_range(0..fold_list.len());
    let random_element = fold_list.get(index).unwrap();
    Ok(random_element.clone())
}

pub async fn update_segment_status(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    segment: &JournalSegment,
    status: SegmentStatus,
) -> Result<(), MetaServiceError> {
    let mut new_segment = segment.clone();
    new_segment.status = status;
    sync_save_segment_info(raft_manager, &new_segment).await?;

    cache_manager.set_segment(&new_segment);

    Ok(())
}

pub async fn sync_save_segment_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &JournalSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalSetSegment,
        Bytes::copy_from_slice(&Bytes::copy_from_slice(&segment.encode()?)),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &JournalSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalDeleteSegment,
        Bytes::copy_from_slice(&segment.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_save_segment_metadata_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &JournalSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalSetSegmentMetadata,
        Bytes::copy_from_slice(&segment.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_metadata_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &JournalSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::JournalDeleteSegmentMetadata,
        Bytes::copy_from_slice(&segment.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

#[cfg(test)]
mod tests {
    use super::calc_node_fold;
    use crate::core::cache::CacheManager;
    use common_base::tools::now_second;
    use metadata_struct::meta::node::BrokerNode;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::sync::Arc;

    #[tokio::test]
    async fn calc_node_fold_test() {
        let rocksdb_engine_handler = test_rocksdb_instance();
        let cache_manager = Arc::new(CacheManager::new(rocksdb_engine_handler));

        let node = BrokerNode {
            roles: Vec::new(),
            register_time: now_second(),
            start_time: now_second(),
            node_id: 1,
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
            storage_fold: vec!["../data/d1".to_string(), "../data/d2".to_string()],
            extend: Vec::new(),
        };
        cache_manager.add_broker_node(node);
        let res = calc_node_fold(&cache_manager, 1).unwrap();
        assert!(!res.is_empty())
    }

    #[tokio::test]
    async fn create_segment_test() {}
}
