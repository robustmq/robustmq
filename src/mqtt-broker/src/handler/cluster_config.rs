use tonic::Status;
use common_base::error::common::CommonError;
use crate::handler::cache::CacheManager;
use crate::storage::cluster::ClusterStorage;
use metadata_struct::mqtt::cluster::MqttClusterDynamicConfig;

impl CacheManager {
    pub(super) fn set_cluster_info(&self, cluster: MqttClusterDynamicConfig){
        self.cluster_info.insert(self.cluster_name.clone(), cluster);
    }

    pub fn enable_slow_sub(&self, is_enable: bool) -> Result<(), CommonError>{
        // save in cache
        let mut dynamic_config = self.get_cluster_info();
        dynamic_config.slow.enable = is_enable;
        self.set_cluster_info(dynamic_config.clone());

        // save in storage
        let client_pool = self.client_pool.clone();
        let cluster_name = self.cluster_name.clone();
        let cluster_storage = ClusterStorage::new(client_pool);
        cluster_storage.set_cluster_config(cluster_name, dynamic_config.clone())?
    }
}