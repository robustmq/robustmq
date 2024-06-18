use clients::placement::placement::call::{
    delete_resource_config, get_resource_config, heartbeat, register_node, set_resource_config,
    un_register_node,
};
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{debug, error, info},
    tools::get_local_ip,
};
use metadata_struct::mqtt::cluster::MQTTCluster;
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
            mqtt4_addr: format!("{}:{}", local_ip, config.mqtt.mqtt4_port),
            mqtt4s_addr: format!("{}:{}", local_ip, config.mqtt.mqtts4_port),
            mqtt5_addr: format!("{}:{}", local_ip, config.mqtt.mqtt5_port),
            mqtt5s_addr: format!("{}:{}", local_ip, config.mqtt.mqtts5_port),
            websocket_addr: format!("{}:{}", local_ip, config.mqtt.websocket_port),
            websockets_addr: format!("{}:{}", local_ip, config.mqtt.websockets_port),
        };
        req.extend_info = serde_json::to_string(&node).unwrap();

        match register_node(
            self.client_poll.clone(),
            config.placement.server.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {
                info(format!(
                    "Node {} has been successfully registered",
                    config.broker_id
                ));
            }
            Err(e) => {
                panic!("{}", e.to_string())
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
            config.placement.server.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {
                info(format!("Node {} exits successfully", config.broker_id));
            }
            Err(e) => error(e.to_string()),
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
            config.placement.server.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {
                debug(format!(
                    "Node {} successfully reports the heartbeat communication",
                    config.broker_id
                ));
            }
            Err(e) => error(e.to_string()),
        }
    }

    pub async fn set_cluster_config(&self, cluster: MQTTCluster) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(config.cluster_name.clone());
        let request = SetResourceConfigRequest {
            cluster_name: config.cluster_name.clone(),
            resources,
            config: cluster.encode(),
        };

        match set_resource_config(
            self.client_poll.clone(),
            config.placement.server.clone(),
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

    pub async fn delete_cluster_config(&self) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(config.cluster_name.clone());
        let request = DeleteResourceConfigRequest {
            cluster_name: config.cluster_name.clone(),
            resources,
        };

        match delete_resource_config(
            self.client_poll.clone(),
            config.placement.server.clone(),
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

    pub async fn get_cluster_config(&self) -> Result<Option<MQTTCluster>, RobustMQError> {
        let config = broker_mqtt_conf();
        let resources = self.cluster_config_resources(config.cluster_name.clone());
        let request = GetResourceConfigRequest {
            cluster_name: config.cluster_name.clone(),
            resources,
        };

        match get_resource_config(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(data) => {
                if data.config.is_empty() {
                    return Ok(None);
                } else {
                    match serde_json::from_str::<MQTTCluster>(&data.config) {
                        Ok(data) => {
                            return Ok(Some(data));
                        }
                        Err(e) => {
                            return Err(RobustMQError::CommmonError(e.to_string()));
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
