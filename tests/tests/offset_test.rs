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
    use common_base::uuid::unique_id;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use common_group::{manager::OffsetManager, storage::start_offset_sync_task};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use std::{collections::HashMap, sync::Arc, time::Duration};
    use tokio::{sync::broadcast, time::sleep};

    #[tokio::test]
    async fn offset_manager_offset_storage() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let client_pool = Arc::new(ClientPool::new(3));
        let offset_manager = Arc::new(OffsetManager::new(client_pool));

        // start meta service
        let (stop_send, _) = broadcast::channel(2);

        let raw_offset_manager = offset_manager.clone();
        tokio::spawn(async move {
            start_offset_sync_task(raw_offset_manager, stop_send).await;
        });

        let group_name = unique_id();
        let mut offset = HashMap::new();
        offset.insert("k1".to_string(), 3);
        offset_manager
            .commit_offset(DEFAULT_TENANT, &group_name, &offset)
            .await
            .unwrap();

        sleep(Duration::from_secs(3)).await;
        let rep_offset = offset_manager
            .get_offset(DEFAULT_TENANT, &group_name)
            .await
            .unwrap();
        assert_eq!(rep_offset.len(), 1);
        let o1 = rep_offset.first().unwrap();
        assert_eq!(o1.offset, 3);
    }
}
