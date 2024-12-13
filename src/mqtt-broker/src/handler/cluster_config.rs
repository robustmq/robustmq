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

use metadata_struct::mqtt::cluster::{MqttClusterDynamicConfig, MqttClusterDynamicSlowSub};

use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::cluster::ClusterStorage;

/// This section primarily implements cache management for cluster-related configuration operations.
/// Through this implementation, we can retrieve configuration information within the cluster
/// and set corresponding cluster configuration attributes.
impl CacheManager {
    pub(super) fn set_cluster_info(&self, cluster: MqttClusterDynamicConfig) {
        self.cluster_info.insert(self.cluster_name.clone(), cluster);
    }

    pub fn get_cluster_info(&self) -> MqttClusterDynamicConfig {
        if let Some(cluster) = self.cluster_info.get(&self.cluster_name) {
            return cluster.clone();
        }
        MqttClusterDynamicConfig::new()
    }

    pub async fn enable_slow_sub(&self, is_enable: bool) -> Result<(), MqttBrokerError> {
        // save in cache
        let mut dynamic_config = self.get_cluster_info();
        dynamic_config.slow.enable = is_enable;
        self.set_cluster_info(dynamic_config.clone());

        // save in storage
        let client_pool = self.client_pool.clone();
        let cluster_storage = ClusterStorage::new(client_pool);
        cluster_storage
            .set_cluster_config(&self.cluster_name, dynamic_config)
            .await?;
        Ok(())
    }

    pub fn get_slow_sub_config(&self) -> MqttClusterDynamicSlowSub {
        self.get_cluster_info().slow
    }
}
