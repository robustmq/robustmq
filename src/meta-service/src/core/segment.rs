use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use bytes::Bytes;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::storage::segment::{EngineSegment, Replica, SegmentStatus};
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use rand::{thread_rng, Rng};
use std::sync::Arc;

pub async fn create_segment(
    shard_info: &EngineShard,
    cache_manager: &Arc<CacheManager>,
    segment_no: u32,
) -> Result<EngineSegment, MetaServiceError> {
    if let Some(segment) = cache_manager.get_segment(&shard_info.shard_name, segment_no) {
        return Ok(segment);
    }

    let node_list: Vec<BrokerNode> = cache_manager.get_engine_node_list();
    if node_list.len() < shard_info.config.replica_num as usize {
        return Err(MetaServiceError::NotEnoughEngineNodes(
            shard_info.config.replica_num,
            node_list.len() as u32,
        ));
    }

    //todo Get the node copies at random
    let node_ids: Vec<u64> = node_list.iter().map(|raw| raw.node_id).collect();

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

    Ok(EngineSegment {
        shard_name: shard_info.shard_name.clone(),
        leader_epoch: 0,
        status: SegmentStatus::Idle,
        segment_seq: segment_no,
        leader: calc_leader_node(&replicas),
        replicas: replicas.clone(),
        isr: replicas.iter().map(|rep| rep.node_id).collect(),
    })
}

pub fn calc_leader_node(replicas: &[Replica]) -> u64 {
    replicas.first().unwrap().node_id
}

pub fn calc_node_fold(
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
    segment: &EngineSegment,
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
    segment: &EngineSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineSetSegment,
        Bytes::copy_from_slice(&Bytes::copy_from_slice(&segment.encode()?)),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &EngineSegment,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineDeleteSegment,
        Bytes::copy_from_slice(&segment.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_save_segment_metadata_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &EngineSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineSetSegmentMetadata,
        Bytes::copy_from_slice(&segment.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_segment_metadata_info(
    raft_manager: &Arc<MultiRaftManager>,
    segment: &EngineSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineDeleteSegmentMetadata,
        Bytes::copy_from_slice(&segment.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
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
            node_inner_addr: "".to_string(),
            node_ip: "".to_string(),
            storage_fold: vec!["../data/d1".to_string(), "../data/d2".to_string()],
            extend: Vec::new(),
        };
        cache_manager.add_broker_node(node);
        let res = calc_node_fold(&cache_manager, 1).unwrap();
        assert!(!res.is_empty())
    }
}
