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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::controller::call_broker::storage::update_cache_by_set_segment;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::core::segment_meta::{
    create_segment_metadata, sync_delete_segment_metadata_info,
    update_end_timestamp_by_segment_metadata,
};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use bytes::Bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::storage::segment::{EngineSegment, Replica, SegmentStatus};
use metadata_struct::storage::shard::EngineShard;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn create_segment(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_info: &EngineShard,
    segment_seq: u32,
    start_offset: u64,
) -> Result<EngineSegment, MetaServiceError> {
    let segment = if let Some(segment) =
        cache_manager.get_segment(&shard_info.shard_name, segment_seq)
    {
        segment
    } else {
        info!(
            "Creating new segment: shard={}, segment={}, start_offset={}",
            shard_info.shard_name, segment_seq, start_offset
        );

        let segment: EngineSegment = build_segment(shard_info, cache_manager, segment_seq).await?;

        sync_save_segment_info(raft_manager, &segment).await?;
        update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;

        create_segment_metadata(
            cache_manager,
            raft_manager,
            call_manager,
            client_pool,
            &segment,
            start_offset as i64,
        )
        .await?;

        segment
    };
    Ok(segment)
}

pub async fn seal_up_segment(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment: &EngineSegment,
    last_timestamp: u64,
) -> Result<(), MetaServiceError> {
    info!(
        "Sealing up segment: shard={}, segment={}, last_timestamp={}",
        segment.shard_name, segment.segment_seq, last_timestamp
    );

    update_segment_status(
        cache_manager,
        call_manager,
        raft_manager,
        client_pool,
        &segment.shard_name,
        segment.segment_seq,
        SegmentStatus::SealUp,
    )
    .await?;

    update_end_timestamp_by_segment_metadata(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        &segment.shard_name,
        segment.segment_seq,
        last_timestamp,
    )
    .await?;

    Ok(())
}

pub async fn delete_segment_by_real(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    segment: &EngineSegment,
) -> Result<(), MetaServiceError> {
    info!(
        "Deleting segment: shard={}, segment={}",
        segment.shard_name, segment.segment_seq
    );

    sync_delete_segment_info(raft_manager, segment).await?;

    if let Some(meta) = cache_manager.get_segment_meta(&segment.shard_name, segment.segment_seq) {
        sync_delete_segment_metadata_info(raft_manager, &meta).await?;
    }
    Ok(())
}

async fn build_segment(
    shard_info: &EngineShard,
    cache_manager: &Arc<CacheManager>,
    segment_no: u32,
) -> Result<EngineSegment, MetaServiceError> {
    if let Some(segment) = cache_manager.get_segment(&shard_info.shard_name, segment_no) {
        return Ok(segment);
    }

    let node_list: Vec<BrokerNode> = cache_manager.get_engine_node_list();
    let replica_num = shard_info.config.replica_num as usize;
    println!("node_list:{:?}", node_list);
    if node_list.len() < replica_num {
        return Err(MetaServiceError::NotEnoughEngineNodes(
            "CreateSegment".to_string(),
            shard_info.config.replica_num,
            node_list.len() as u32,
        ));
    }

    let mut rng = thread_rng();
    let selected_nodes: Vec<&BrokerNode> =
        node_list.choose_multiple(&mut rng, replica_num).collect();

    let mut replicas = Vec::new();
    for (i, node) in selected_nodes.iter().enumerate() {
        let fold = calc_node_fold(cache_manager, node.node_id)?;
        replicas.push(Replica {
            replica_seq: i as u64,
            node_id: node.node_id,
            fold,
        });
    }

    if replicas.len() != replica_num {
        return Err(MetaServiceError::NumberOfReplicasIsIncorrect(
            shard_info.config.replica_num,
            replicas.len(),
        ));
    }

    let leader = calc_leader_node(&replicas)?;
    let isr: Vec<u64> = replicas.iter().map(|rep| rep.node_id).collect();

    Ok(EngineSegment {
        shard_name: shard_info.shard_name.clone(),
        leader_epoch: 0,
        status: SegmentStatus::Write,
        segment_seq: segment_no,
        leader,
        replicas,
        isr,
    })
}

pub async fn update_segment_status(
    cache_manager: &Arc<CacheManager>,
    broker_call_manager: &Arc<BrokerCallManager>,
    raft_manager: &Arc<MultiRaftManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment_seq: u32,
    status: SegmentStatus,
) -> Result<(), MetaServiceError> {
    if let Some(segment_list) = cache_manager.segment_list.get(shard_name) {
        if let Some(mut segment) = segment_list.get_mut(&segment_seq) {
            info!(
                "Updating segment status: shard={}, segment={}, old_status={:?}, new_status={:?}",
                shard_name, segment_seq, segment.status, status
            );

            segment.status = status;
        }
    }

    if let Some(segment) = cache_manager.get_segment(shard_name, segment_seq) {
        sync_save_segment_info(raft_manager, &segment).await?;
        update_cache_by_set_segment(broker_call_manager, client_pool, segment).await?;
    }

    Ok(())
}

async fn sync_save_segment_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &EngineSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineSetSegment,
        Bytes::from(segment.encode()?),
    );

    raft_manager
        .write_metadata(data)
        .await?
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;

    Ok(())
}

async fn sync_delete_segment_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &EngineSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineDeleteSegment,
        Bytes::from(segment.encode()?),
    );

    raft_manager
        .write_metadata(data)
        .await?
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;

    Ok(())
}

fn calc_leader_node(replicas: &[Replica]) -> Result<u64, MetaServiceError> {
    replicas.first().map(|rep| rep.node_id).ok_or_else(|| {
        warn!("Cannot calculate leader node: replica list is empty");
        MetaServiceError::CommonError("Replica list is empty".to_string())
    })
}

fn calc_node_fold(
    cache_manager: &Arc<CacheManager>,
    node_id: u64,
) -> Result<String, MetaServiceError> {
    let node = cache_manager
        .get_broker_node(node_id)
        .ok_or_else(|| MetaServiceError::NodeDoesNotExist(node_id))?;

    if node.storage_fold.is_empty() {
        return Err(MetaServiceError::CommonError(format!(
            "Node {} has no storage folders configured",
            node_id
        )));
    }

    let fold_list = &node.storage_fold;
    let mut rng = thread_rng();
    let random_element = fold_list.choose(&mut rng).ok_or_else(|| {
        MetaServiceError::CommonError("Failed to select storage folder".to_string())
    })?;

    Ok(random_element.clone())
}

#[cfg(test)]
mod tests {
    use crate::core::{cache::CacheManager, segment::calc_node_fold};
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
            grpc_addr: "".to_string(),
            node_ip: "".to_string(),
            storage_fold: vec!["../data/d1".to_string(), "../data/d2".to_string()],
            extend: Vec::new(),
            ..Default::default()
        };
        cache_manager.add_broker_node(node);
        let res = calc_node_fold(&cache_manager, 1).unwrap();
        assert!(!res.is_empty())
    }
}
