use crate::metadata::node::Node;
use clients::{
    placement_center::placement::{heartbeat, register_node, unregister_node},
    ClientPool,
};
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{debug_eninge, error_engine, info_meta},
    tools::get_local_ip,
};
use protocol::placement_center::generate::{
    common::ClusterType,
    placement::{HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest},
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

pub mod report;

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

    let mut res_err = None;
    for addr in config.placement_center.clone() {
        match register_node(client_poll.clone(), addr, req.clone()).await {
            Ok(_) => {
                info_meta(&format!(
                    "Node {} has been successfully registered",
                    config.broker_id
                ));
                break;
            }
            Err(e) => {
                res_err = Some(e);
            }
        }
    }
    if !res_err.is_none() {
        let info = res_err.unwrap().to_string();
        panic!("{}", info);
    }
}

pub async fn unregister_broker_node(client_poll: Arc<Mutex<ClientPool>>) {
    let config = broker_mqtt_conf();
    let mut req = UnRegisterNodeRequest::default();
    req.cluster_type = ClusterType::MqttBrokerServer.into();
    req.cluster_name = config.cluster_name.clone();
    req.node_id = config.broker_id;

    let mut res_err = None;
    for addr in config.placement_center.clone() {
        match unregister_node(client_poll.clone(), addr, req.clone()).await {
            Ok(_) => {
                info_meta(&format!("Node {} exits successfully", config.broker_id));
                break;
            }
            Err(e) => {
                res_err = Some(e);
            }
        }
    }
    if !res_err.is_none() {
        let info = res_err.unwrap().to_string();
        panic!("{}", info);
    }
}

pub async fn report_heartbeat(client_poll: Arc<Mutex<ClientPool>>) {
    
    time::sleep(Duration::from_millis(5000)).await;
    loop {
        let config = broker_mqtt_conf();
        let mut req = HeartbeatRequest::default();
        req.cluster_name = config.cluster_name.clone();
        req.cluster_type = ClusterType::MqttBrokerServer.into();
        req.node_id = config.broker_id;
        let mut res_err = None;
        for addr in config.placement_center.clone() {
            match heartbeat(client_poll.clone(), addr, req.clone()).await {
                Ok(_) => {
                    debug_eninge(format!(
                        "Node {} successfully reports the heartbeat communication",
                        config.broker_id
                    ));
                    break;
                }
                Err(e) => {
                    res_err = Some(e);
                }
            }
        }
        if !res_err.is_none() {
            error_engine(res_err.unwrap().to_string());
        }
        time::sleep(Duration::from_millis(1000)).await;
    }
}
