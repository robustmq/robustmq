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

use clients::placement::placement::call::{
    delete_resource_config, get_resource_config, heartbeat, register_node, set_resource_config,
    un_register_node,
};
use clients::poll::ClientPool;
use common_base::error::common::CommonError;
use common_base::{config::broker_mqtt::broker_mqtt_conf, tools::get_local_ip};
use log::{error, info};
use metadata_struct::mqtt::cluster::MQTTClusterDynamicConfig;
use metadata_struct::mqtt::node_extend::MQTTNodeExtend;
use protocol::placement_center::generate::placement::{
    DeleteResourceConfigRequest, GetResourceConfigRequest, SetResourceConfigRequest,
};
use protocol::placement_center::generate::{
    common::ClusterType,
    placement::{HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest},
};
use std::sync::Arc;

pub struct ClusterStorage {
    client_poll: Arc<ClientPool>,
}

impl ClusterStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return ClusterStorage { client_poll };
    }

    pub async fn register_node(&self) {
        let config = broker_mqtt_conf();
        let local_ip = get_local_ip();
        let mut req = RegisterNodeRequest::default();
        req.cluster_type = ClusterType::MqttBrokerServer.into();
        req.cluster_name = config.cluster_name.clone();
        req.node_id = config.broker_id;
        req.node_ip = local_ip.clone();
        req.node_inner_addr = format!("{}:{}", local_ip, config.grpc_port);

        //  mqtt broker extend info
        let node = MQTTNodeExtend {
            grpc_addr: format!("{}:{}", local_ip, config.grpc_port),
            http_addr: format!("{}:{}", local_ip, config.http_port),
            mqtt_addr: format!("{}:{}", local_ip, config.network.tcp_port),
            mqtts_addr: format!("{}:{}", local_ip, config.network.tcps_port),
            websocket_addr: format!("{}:{}", local_ip, config.network.websocket_port),
            websockets_addr: format!("{}:{}", local_ip, config.network.websockets_port),
            quic_addr: format!("{}:{}", local_ip, config.network.quic_port),
        };
        req.extend_info = serde_json::to_string(&node).unwrap();

        match register_node(
            self.client_poll.clone(),
            config.placement_center.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {
                info!("Node {} has been successfully registered", config.broker_id);
            }
            Err(e) => {
                panic!("Register node fail,{}", e.to_string())
            }
        }
    }

    pub async fn unregister_node(&self) {
        let config = broker_mqtt_conf();
        let mut req = UnRegisterNodeRequest::default();
        req.cluster_type = ClusterType::MqttBrokerServer.into();
        req.cluster_name = config.cluster_name.clone();
        req.node_id = config.broker_id;

        match un_register_node(
            self.client_poll.clone(),
            config.placement_center.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {
                info!("Node {} exits successfully", config.broker_id);
            }
            Err(e) => error!("{}", e),
        }
    }

    pub async fn heartbeat(&self) {
        let config = broker_mqtt_conf();
        let mut req = HeartbeatRequest::default();
        req.cluster_name = config.cluster_name.clone();
        req.cluster_type = ClusterType::MqttBrokerServer.into();
        req.node_id = config.broker_id;

        match heartbeat(
            self.client_poll.clone(),
            config.placement_center.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }
    }

    pub async fn set_cluster_config(
        &self,
        cluster_name: String,
        cluster: MQTTClusterDynamicConfig,
    ) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(cluster_name.clone());
        let request = SetResourceConfigRequest {
            cluster_name: cluster_name.clone(),
            resources,
            config: cluster.encode(),
        };

        match set_resource_config(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn delete_cluster_config(&self, cluster_name: String) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(cluster_name.clone());
        let request = DeleteResourceConfigRequest {
            cluster_name: cluster_name.clone(),
            resources,
        };

        match delete_resource_config(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn get_cluster_config(
        &self,
        cluster_name: String,
    ) -> Result<Option<MQTTClusterDynamicConfig>, CommonError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(cluster_name.clone());
        let request = GetResourceConfigRequest {
            cluster_name: cluster_name.clone(),
            resources,
        };

        match get_resource_config(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(data) => {
                if data.config.is_empty() {
                    return Ok(None);
                } else {
                    match serde_json::from_slice::<MQTTClusterDynamicConfig>(&data.config) {
                        Ok(data) => {
                            return Ok(Some(data));
                        }
                        Err(e) => {
                            return Err(CommonError::CommmonError(e.to_string()));
                        }
                    }
                }
            }
            Err(e) => return Err(e),
        }
    }

    fn cluster_config_resources(&self, cluster_name: String) -> Vec<String> {
        return vec!["cluster".to_string(), cluster_name];
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::cluster::ClusterStorage;
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use metadata_struct::mqtt::cluster::MQTTClusterDynamicConfig;
    use std::sync::Arc;

    #[tokio::test]
    async fn cluster_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);

        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));
        let cluster_storage = ClusterStorage::new(client_poll);

        let cluster_name = "robust_test".to_string();
        let mut cluster = MQTTClusterDynamicConfig::default();
        cluster.protocol.topic_alias_max = 999;
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
