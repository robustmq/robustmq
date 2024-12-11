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

use std::sync::Arc;

use common_base::config::broker_mqtt::{broker_mqtt_conf, BrokerMqttConfig};
use common_base::error::common::CommonError;
use common_base::tools::get_local_ip;
use grpc_clients::placement::inner::call::{
    delete_resource_config, get_resource_config, heartbeat, node_list, register_node,
    set_resource_config, unregister_node,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::cluster::MqttClusterDynamicConfig;
use metadata_struct::mqtt::node_extend::MqttNodeExtend;
use metadata_struct::placement::node::BrokerNode;
use protocol::placement_center::placement_center_inner::{
    ClusterType, DeleteResourceConfigRequest, GetResourceConfigRequest, HeartbeatRequest,
    NodeListRequest, RegisterNodeRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
};

pub struct ClusterStorage {
    client_pool: Arc<ClientPool>,
}

impl ClusterStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        ClusterStorage { client_pool }
    }

    pub async fn node_list(&self) -> Result<Vec<BrokerNode>, CommonError> {
        let conf = broker_mqtt_conf();
        let request = NodeListRequest {
            cluster_name: conf.cluster_name.clone(),
        };

        let reply = node_list(self.client_pool.clone(), &conf.placement_center, request).await?;

        let mut node_list: Vec<BrokerNode> = Vec::new();
        for node in reply.nodes {
            match serde_json::from_slice::<BrokerNode>(&node) {
                Ok(data) => node_list.push(data),
                Err(e) => {
                    return Err(CommonError::CommonError(format!("Retrieving cluster Node list, parsing Node information failed, error message :{}", e)));
                }
            }
        }
        Ok(node_list)
    }

    pub async fn register_node(&self, config: &BrokerMqttConfig) -> Result<(), CommonError> {
        let local_ip = get_local_ip();

        let node = MqttNodeExtend {
            grpc_addr: format!("{}:{}", local_ip, config.grpc_port),
            http_addr: format!("{}:{}", local_ip, config.http_port),
            mqtt_addr: format!("{}:{}", local_ip, config.network.tcp_port),
            mqtts_addr: format!("{}:{}", local_ip, config.network.tcps_port),
            websocket_addr: format!("{}:{}", local_ip, config.network.websocket_port),
            websockets_addr: format!("{}:{}", local_ip, config.network.websockets_port),
            quic_addr: format!("{}:{}", local_ip, config.network.quic_port),
        };
        let req = RegisterNodeRequest {
            cluster_type: ClusterType::MqttBrokerServer.into(),
            cluster_name: config.cluster_name.clone(),
            node_ip: local_ip.clone(),
            node_id: config.broker_id,
            node_inner_addr: format!("{}:{}", local_ip, config.grpc_port),
            extend_info: serde_json::to_string(&node).unwrap(),
        };

        register_node(
            self.client_pool.clone(),
            &config.placement_center,
            req.clone(),
        )
        .await?;

        Ok(())
    }

    pub async fn unregister_node(&self, config: &BrokerMqttConfig) -> Result<(), CommonError> {
        let req = UnRegisterNodeRequest {
            cluster_type: ClusterType::MqttBrokerServer.into(),
            cluster_name: config.cluster_name.clone(),
            node_id: config.broker_id,
        };

        unregister_node(
            self.client_pool.clone(),
            &config.placement_center,
            req.clone(),
        )
        .await?;
        Ok(())
    }

    pub async fn heartbeat(&self) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let req = HeartbeatRequest {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::MqttBrokerServer.into(),
            node_id: config.broker_id,
        };

        heartbeat(
            self.client_pool.clone(),
            &config.placement_center,
            req.clone(),
        )
        .await?;

        Ok(())
    }

    pub async fn set_cluster_config(
        &self,
        cluster_name: &str,
        cluster: MqttClusterDynamicConfig,
    ) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(cluster_name.to_string());
        let request = SetResourceConfigRequest {
            cluster_name: cluster_name.to_string(),
            resources,
            config: cluster.encode(),
        };

        set_resource_config(self.client_pool.clone(), &config.placement_center, request).await?;

        Ok(())
    }

    pub async fn delete_cluster_config(&self, cluster_name: &str) -> Result<(), CommonError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(cluster_name.to_string());
        let request = DeleteResourceConfigRequest {
            cluster_name: cluster_name.to_string(),
            resources,
        };

        delete_resource_config(self.client_pool.clone(), &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn get_cluster_config(
        &self,
        cluster_name: &str,
    ) -> Result<Option<MqttClusterDynamicConfig>, CommonError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(cluster_name.to_string());
        let request = GetResourceConfigRequest {
            cluster_name: cluster_name.to_string(),
            resources,
        };

        match get_resource_config(self.client_pool.clone(), &config.placement_center, request).await
        {
            Ok(data) => {
                if data.config.is_empty() {
                    Ok(None)
                } else {
                    match serde_json::from_slice::<MqttClusterDynamicConfig>(&data.config) {
                        Ok(data) => Ok(Some(data)),
                        Err(e) => Err(CommonError::CommonError(e.to_string())),
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    fn cluster_config_resources(&self, cluster_name: String) -> Vec<String> {
        vec!["cluster".to_string(), cluster_name]
    }
}
