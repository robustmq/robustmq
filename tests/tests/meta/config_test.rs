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
    use broker_core::cluster::ClusterStorage;
    use common_config::{
        broker::{default_broker_config, init_broker_conf_by_config},
        config::MqttProtocolConfig,
    };
    use grpc_clients::pool::ClientPool;
    use std::sync::Arc;

    #[tokio::test]
    async fn cluster_config_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let cluster_storage = ClusterStorage::new(client_pool);

        let protocol = MqttProtocolConfig {
            topic_alias_max: 999,
            ..Default::default()
        };
        let resource = "MqttProtocol";
        cluster_storage
            .set_dynamic_config(resource, protocol.encode())
            .await
            .unwrap();

        let result = cluster_storage.get_dynamic_config(resource).await.unwrap();
        let result: MqttProtocolConfig = serde_json::from_slice(&result).unwrap();
        assert_eq!(result.topic_alias_max, 999);

        cluster_storage
            .delete_dynamic_config(resource)
            .await
            .unwrap();

        let result = cluster_storage.get_dynamic_config(resource).await.unwrap();
        assert!(result.is_empty());
    }
}
