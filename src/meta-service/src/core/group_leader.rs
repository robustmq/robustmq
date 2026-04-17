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
use crate::{core::cache::MetaCacheManager, storage::common::share_group::ShareGroupStorage};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{collections::HashMap, sync::Arc};

pub async fn get_group_leader(
    _raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    tenant: &str,
    group_name: &str,
) -> Result<u64, MetaServiceError> {
    let cache_key = format!("{}/{}", tenant, group_name);
    if let Some(leader) = cache_manager.group_leader.get(&cache_key) {
        if cache_manager.node_list.contains_key(&leader.broker_id) {
            return Ok(leader.broker_id);
        }
    }

    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
    let list = storage.list_by_tenant(tenant)?;
    if let Some(leader) = list.get(group_name) {
        if cache_manager.node_list.contains_key(&leader.broker_id) {
            return Ok(leader.broker_id);
        }
    }

    Err(MetaServiceError::ShareGroupDoesNotExist(cache_key))
}

pub async fn generate_group_leader(
    cache_manager: &Arc<MetaCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    tenant: &str,
) -> Result<u64, MetaServiceError> {
    let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
    let list = storage.list_by_tenant(tenant)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::meta::node::BrokerNode;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup() -> (Arc<MetaCacheManager>, Arc<RocksDBEngine>) {
        let config = default_broker_config();
        init_broker_conf_by_config(config);
        let db = test_rocksdb_instance();
        let cache_manager = Arc::new(MetaCacheManager::new(db.clone()));
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

        let tenant = "test_tenant";
        let storage = ShareGroupStorage::new(rocksdb_engine_handler.clone());
        use metadata_struct::mqtt::share_group::ShareGroupLeader;
        storage
            .save(ShareGroupLeader {
                tenant: tenant.to_string(),
                group_name: "g1".to_string(),
                broker_id: 1,
                ..Default::default()
            })
            .unwrap();
        storage
            .save(ShareGroupLeader {
                tenant: tenant.to_string(),
                group_name: "g2".to_string(),
                broker_id: 1,
                ..Default::default()
            })
            .unwrap();
        storage
            .save(ShareGroupLeader {
                tenant: tenant.to_string(),
                group_name: "g3".to_string(),
                broker_id: 2,
                ..Default::default()
            })
            .unwrap();

        let target = generate_group_leader(&cache_manager, &rocksdb_engine_handler, tenant)
            .await
            .unwrap();
        assert_eq!(target, 3);
    }
}
