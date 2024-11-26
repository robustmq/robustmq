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
    use std::sync::Arc;

    use common_base::config::broker_mqtt::{broker_mqtt_conf, init_broker_mqtt_conf_by_path};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::cluster::{
        MqttClusterDynamicConfig, MqttClusterDynamicConfigProtocol,
    };
    use mqtt_broker::storage::cluster::ClusterStorage;

    #[tokio::test]
    async fn cluster_node_test() {
        let path = format!("{}/../config/mqtt-server.toml", env!("CARGO_MANIFEST_DIR"));
        init_broker_mqtt_conf_by_path(&path);

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let cluster_storage = ClusterStorage::new(client_pool);

        let mut config = broker_mqtt_conf().clone();
        config.broker_id = 1234u64;
        cluster_storage.register_node(&config).await.unwrap();

        let node_list = cluster_storage.node_list().await.unwrap();
        let register_node_exist = node_list
            .iter()
            .any(|node| node.node_id == config.broker_id);
        assert!(register_node_exist);

        cluster_storage.unregister_node(&config).await.unwrap();

        let node_list_after_unregister = cluster_storage.node_list().await.unwrap();
        let unregister_node_exist = node_list_after_unregister
            .iter()
            .any(|node| node.node_id == config.broker_id);
        assert!(!unregister_node_exist);
    }

    #[tokio::test]
    async fn cluster_config_test() {
        let path = format!("{}/../config/mqtt-server.toml", env!("CARGO_MANIFEST_DIR"));
        init_broker_mqtt_conf_by_path(&path);

        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let cluster_storage = ClusterStorage::new(client_pool);

        let cluster_name = "robust_test".to_string();
        let cluster = MqttClusterDynamicConfig {
            protocol: MqttClusterDynamicConfigProtocol {
                topic_alias_max: 999,
                ..Default::default()
            },
            ..Default::default()
        };
        cluster_storage
            .set_cluster_config(cluster_name.clone(), cluster)
            .await
            .unwrap();

        let result = cluster_storage
            .get_cluster_config(cluster_name.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.protocol.topic_alias_max, 999);

        cluster_storage
            .delete_cluster_config(cluster_name.clone())
            .await
            .unwrap();

        let result = cluster_storage
            .get_cluster_config(cluster_name.clone())
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
