use crate::metadata::node::Node;
use clients::{
    placement::placement::call::{heartbeat, register_node, un_register_node},
    ClientPool,
};
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{debug, info},
    tools::get_local_ip,
};
use protocol::placement_center::generate::{
    common::ClusterType,
    placement::{HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest},
};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, Mutex},
    time,
};

pub mod heartbeat_manager;
pub mod keep_alive;
pub mod report;

pub const HEART_CONNECT_SHARD_HASH_NUM: u64 = 20;

pub async fn register_broker_node(client_poll: Arc<Mutex<ClientPool>>) {
    let config = broker_mqtt_conf();
    let mut req = RegisterNodeRequest::default();
    req.cluster_type = ClusterType::MqttBrokerServer.into();
    req.cluster_name = config.cluster_name.clone();
    req.node_id = config.broker_id;
    req.node_ip = get_local_ip();
    req.node_port = config.grpc_port;

    //  mqtt broker extend info
    let node = Node {
        mqtt4_enable: true,
        mqtt5_enable: true,
    };
    req.extend_info = serde_json::to_string(&node).unwrap();

    match register_node(
        client_poll.clone(),
        config.placement_center.clone(),
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

pub async fn unregister_broker_node(client_poll: Arc<Mutex<ClientPool>>) {
    let config = broker_mqtt_conf();
    let mut req = UnRegisterNodeRequest::default();
    req.cluster_type = ClusterType::MqttBrokerServer.into();
    req.cluster_name = config.cluster_name.clone();
    req.node_id = config.broker_id;

    match un_register_node(
        client_poll.clone(),
        config.placement_center.clone(),
        req.clone(),
    )
    .await
    {
        Ok(_) => {
            info(format!("Node {} exits successfully", config.broker_id));
        }
        Err(e) => {
            panic!("{}", e.to_string())
        }
    }
}

pub async fn report_heartbeat(
    client_poll: Arc<Mutex<ClientPool>>,
    mut stop_send: broadcast::Receiver<bool>,
) {
    time::sleep(Duration::from_millis(5000)).await;
    loop {
        match stop_send.try_recv() {
            Ok(flag) => {
                if flag {
                    info("ReportClusterHeartbeat thread stopped successfully".to_string());
                    break;
                }
            }
            Err(_) => {}
        }
        let config = broker_mqtt_conf();
        let mut req = HeartbeatRequest::default();
        req.cluster_name = config.cluster_name.clone();
        req.cluster_type = ClusterType::MqttBrokerServer.into();
        req.node_id = config.broker_id;

        match heartbeat(
            client_poll.clone(),
            config.placement_center.clone(),
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
            Err(e) => {
                panic!("{}", e.to_string())
            }
        }
    }
}
