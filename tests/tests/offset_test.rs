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

#[cfg(test)]
mod tests {
    use common_base::tools::unique_id;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::storage::adapter_offset::AdapterOffsetStrategy;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::{collections::HashMap, sync::Arc, time::Duration};
    use storage_adapter::offset::OffsetManager;
    use tokio::{sync::broadcast, time::sleep};

    #[tokio::test]
    async fn offset_manager_storage() {
        let mut config = default_broker_config();
        config.storage_offset.enable_cache = false;
        init_broker_conf_by_config(config.clone());

        let rocksdb_engine_handler = test_rocksdb_instance();
        let client_pool = Arc::new(ClientPool::new(3));
        let offset_manager = OffsetManager::new(client_pool, rocksdb_engine_handler);
        let group_name = unique_id();
        let mut offset = HashMap::new();
        offset.insert("k1".to_string(), 3);
        offset_manager
            .commit_offset(&group_name, &offset)
            .await
            .unwrap();

        let rep_offset = offset_manager
            .get_offset(&group_name, AdapterOffsetStrategy::Earliest)
            .await
            .unwrap();
        assert_eq!(rep_offset.len(), 1);
        let o1 = rep_offset.first().unwrap();
        assert_eq!(o1.offset, 3);
    }

    #[tokio::test]
    async fn offset_manager_offset_storage() {
        let mut config = default_broker_config();
        config.storage_offset.enable_cache = true;
        init_broker_conf_by_config(config.clone());

        let rocksdb_engine_handler = test_rocksdb_instance();
        let client_pool = Arc::new(ClientPool::new(3));
        let offset_manager = OffsetManager::new(client_pool, rocksdb_engine_handler);

        // start meta service
        let (stop_send, _) = broadcast::channel(2);

        let raw_offset_manager = offset_manager.clone();
        tokio::spawn(async move {
            raw_offset_manager.offset_save_thread(stop_send).await;
        });

        let group_name = unique_id();
        let mut offset = HashMap::new();
        offset.insert("k1".to_string(), 3);
        offset_manager
            .commit_offset(&group_name, &offset)
            .await
            .unwrap();

        sleep(Duration::from_secs(2)).await;
        let rep_offset = offset_manager
            .get_offset(&group_name, AdapterOffsetStrategy::Earliest)
            .await
            .unwrap();
        assert_eq!(rep_offset.len(), 1);
        let o1 = rep_offset.first().unwrap();
        assert_eq!(o1.offset, 3);
    }
}
