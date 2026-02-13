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

use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::{core::cache::CacheManager, storage::mqtt::group_leader::MqttGroupLeaderStorage};
use bytes::Bytes;
use common_base::tools::now_second;
use metadata_struct::mqtt::group_leader::MqttGroupLeader;
use protocol::meta::meta_service_mqtt::{GetShareSubLeaderReply, GetShareSubLeaderRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{collections::HashMap, sync::Arc};

pub async fn get_group_leader(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    group_name: &str,
) -> Result<u64, MetaServiceError> {
    if let Some(leader) = cache_manager.group_leader.get(group_name) {
        if cache_manager.node_list.contains_key(&leader.broker_id) {
            return Ok(leader.broker_id);
        }
    }

    let storage = MqttGroupLeaderStorage::new(rocksdb_engine_handler.clone());
    let list = storage.list()?;
    if let Some(leader) = list.get(group_name) {
        if cache_manager.node_list.contains_key(&leader.broker_id) {
            return Ok(leader.broker_id);
        }
    }

    let target_broker_id = generate_group_leader(cache_manager, rocksdb_engine_handler).await?;
    let leader_info = MqttGroupLeader {
        group_name: group_name.to_string(),
        broker_id: target_broker_id,
        create_time: now_second(),
    };
    let data = StorageData::new(
        StorageDataType::MqttSetGroupLeader,
        Bytes::copy_from_slice(&leader_info.encode()?),
    );
    raft_manager.write_mqtt(data).await?;
    storage.save(group_name, target_broker_id)?;

    Ok(target_broker_id)
}

pub async fn generate_group_leader(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<u64, MetaServiceError> {
    let storage = MqttGroupLeaderStorage::new(rocksdb_engine_handler.clone());
    let list = storage.list()?;
    let mut broker_ids: Vec<u64> = cache_manager
        .node_list
        .iter()
        .map(|node| node.node_id)
        .collect();
    broker_ids.sort_unstable();

    if broker_ids.is_empty() {
        return Err(MetaServiceError::NoAvailableBrokerNode);
    }

    let mut leader_count_by_broker = HashMap::new();
    for broker_id in &broker_ids {
        leader_count_by_broker.insert(*broker_id, 0_u64);
    }

    for leader in list.values() {
        if let Some(count) = leader_count_by_broker.get_mut(&leader.broker_id) {
            *count += 1;
        }
    }

    let target_broker_id = broker_ids
        .iter()
        .min_by_key(|broker_id| {
            let count = leader_count_by_broker.get(broker_id).copied().unwrap_or(0);
            (count, **broker_id)
        })
        .copied()
        .ok_or(MetaServiceError::NoAvailableBrokerNode)?;

    Ok(target_broker_id)
}

pub async fn get_share_sub_leader_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetShareSubLeaderRequest,
) -> Result<GetShareSubLeaderReply, MetaServiceError> {
    let leader_broker = get_group_leader(
        raft_manager,
        cache_manager,
        rocksdb_engine_handler,
        &req.group_name,
    )
    .await?;

    match cache_manager.get_broker_node(leader_broker) {
        Some(node) => Ok(GetShareSubLeaderReply {
            broker_id: node.node_id,
            broker_addr: node.node_ip,
            extend_info: node.extend.encode()?,
        }),
        None => Err(MetaServiceError::NoAvailableBrokerNode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::meta::node::BrokerNode;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup() -> (Arc<CacheManager>, Arc<RocksDBEngine>) {
        let config = default_broker_config();
        init_broker_conf_by_config(config);
        let db = test_rocksdb_instance();
        let cache_manager = Arc::new(CacheManager::new(db.clone()));
        (cache_manager, db)
    }

    #[tokio::test]
    async fn test_generate_group_leader_choose_least_loaded_broker() {
        let (cache_manager, rocksdb_engine_handler) = setup();
        cache_manager.add_broker_node(BrokerNode {
            node_id: 1,
            ..Default::default()
        });
        cache_manager.add_broker_node(BrokerNode {
            node_id: 2,
            ..Default::default()
        });
        cache_manager.add_broker_node(BrokerNode {
            node_id: 3,
            ..Default::default()
        });

        let storage = MqttGroupLeaderStorage::new(rocksdb_engine_handler.clone());
        storage.save("g1", 1).unwrap();
        storage.save("g2", 1).unwrap();
        storage.save("g3", 2).unwrap();

        let target = generate_group_leader(&cache_manager, &rocksdb_engine_handler)
            .await
            .unwrap();
        assert_eq!(target, 3);
    }
}
