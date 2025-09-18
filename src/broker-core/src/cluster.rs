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

use crate::cache::BrokerCacheManager;
use common_base::error::common::CommonError;
use common_base::tools::{get_local_ip, now_second};
use common_config::broker::broker_config;
use common_config::config::BrokerConfig;
use grpc_clients::meta::inner::call::{
    cluster_status, delete_resource_config, get_resource_config, heartbeat, node_list,
    register_node, set_resource_config, unregister_node,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::node_extend::{MqttNodeExtend, NodeExtend};
use metadata_struct::placement::node::BrokerNode;
use protocol::meta::placement_center_inner::{
    ClusterStatusRequest, DeleteResourceConfigRequest, GetResourceConfigRequest, HeartbeatRequest,
    NodeListRequest, RegisterNodeRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
};
use std::sync::Arc;

pub struct ClusterStorage {
    client_pool: Arc<ClientPool>,
}

impl ClusterStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        ClusterStorage { client_pool }
    }

    pub async fn place_cluster_status(&self) -> Result<String, CommonError> {
        let request = ClusterStatusRequest {};
        let conf = broker_config();
        let reply = cluster_status(
            &self.client_pool,
            &conf.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(reply.content)
    }

    pub async fn node_list(&self) -> Result<Vec<BrokerNode>, CommonError> {
        let conf = broker_config();
        let request = NodeListRequest {
            cluster_name: conf.cluster_name.clone(),
        };

        let reply = node_list(
            &self.client_pool,
            &conf.get_placement_center_addr(),
            request,
        )
        .await?;

        let mut node_list: Vec<BrokerNode> = Vec::new();
        for node in reply.nodes {
            let data = serde_json::from_slice::<BrokerNode>(&node)?;
            node_list.push(data);
        }

        Ok(node_list)
    }

    pub async fn register_node(
        &self,
        cache_manager: &Arc<BrokerCacheManager>,
        config: &BrokerConfig,
    ) -> Result<BrokerNode, CommonError> {
        let local_ip = get_local_ip();
        let extend = NodeExtend {
            mqtt: MqttNodeExtend {
                grpc_addr: format!("{}:{}", local_ip, config.grpc_port),
                mqtt_addr: format!("{}:{}", local_ip, config.mqtt_server.tcp_port),
                mqtts_addr: format!("{}:{}", local_ip, config.mqtt_server.tls_port),
                websocket_addr: format!("{}:{}", local_ip, config.mqtt_server.websocket_port),
                websockets_addr: format!("{}:{}", local_ip, config.mqtt_server.websockets_port),
                quic_addr: format!("{}:{}", local_ip, config.mqtt_server.quic_port),
            },
        };

        let node = BrokerNode {
            cluster_name: config.cluster_name.clone(),
            roles: config.roles.clone(),
            node_ip: local_ip.clone(),
            node_id: config.broker_id,
            node_inner_addr: format!("{}:{}", local_ip, config.grpc_port),
            extend: extend.encode(),
            start_time: cache_manager.get_start_time(),
            register_time: now_second(),
        };

        let req = RegisterNodeRequest {
            node: node.encode(),
        };
        register_node(
            &self.client_pool,
            &config.get_placement_center_addr(),
            req.clone(),
        )
        .await?;
        Ok(node)
    }

    pub async fn unregister_node(&self, config: &BrokerConfig) -> Result<(), CommonError> {
        let req = UnRegisterNodeRequest {
            cluster_name: config.cluster_name.clone(),
            node_id: config.broker_id,
        };

        unregister_node(
            &self.client_pool,
            &config.get_placement_center_addr(),
            req.clone(),
        )
        .await?;
        Ok(())
    }

    pub async fn heartbeat(&self) -> Result<(), CommonError> {
        let config = broker_config();
        let req = HeartbeatRequest {
            cluster_name: config.cluster_name.clone(),
            node_id: config.broker_id,
        };

        heartbeat(
            &self.client_pool,
            &config.get_placement_center_addr(),
            req.clone(),
        )
        .await?;

        Ok(())
    }

    pub async fn set_dynamic_config(
        &self,
        cluster_name: &str,
        resource: &str,
        data: Vec<u8>,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let resources = self.dynamic_config_resources(cluster_name, resource);
        let request = SetResourceConfigRequest {
            cluster_name: cluster_name.to_string(),
            resources,
            config: data,
        };

        set_resource_config(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;

        Ok(())
    }

    pub async fn delete_dynamic_config(
        &self,
        cluster_name: &str,
        resource: &str,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let resources = self.dynamic_config_resources(cluster_name, resource);
        let request = DeleteResourceConfigRequest {
            cluster_name: cluster_name.to_string(),
            resources,
        };

        delete_resource_config(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn get_dynamic_config(
        &self,
        cluster_name: &str,
        resource: &str,
    ) -> Result<Vec<u8>, CommonError> {
        let config = broker_config();
        let resources = self.dynamic_config_resources(cluster_name, resource);
        let request = GetResourceConfigRequest {
            cluster_name: cluster_name.to_string(),
            resources,
        };

        let reply = get_resource_config(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(reply.config)
    }

    fn dynamic_config_resources(&self, cluster_name: &str, resource: &str) -> Vec<String> {
        vec![
            "cluster".to_string(),
            cluster_name.to_string(),
            resource.to_string(),
        ]
    }
}
