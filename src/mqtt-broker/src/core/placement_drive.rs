use clients::{
    placement::placement::call::{heartbeat, register_node, un_register_node},
    poll::ClientPool,
};
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{debug, error, info},
    tools::get_local_ip,
};
use metadata_struct::mqtt::node_extend::MQTTNodeExtend;
use protocol::placement_center::generate::{
    common::ClusterType,
    placement::{HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest},
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::broadcast, time};

pub async fn register_broker_node(client_poll: Arc<ClientPool>) {
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
        client_poll.clone(),
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

pub async fn unregister_broker_node(client_poll: Arc<ClientPool>) {
    let config = broker_mqtt_conf();
    let mut req = UnRegisterNodeRequest::default();
    req.cluster_type = ClusterType::MqttBrokerServer.into();
    req.cluster_name = config.cluster_name.clone();
    req.node_id = config.broker_id;

    match un_register_node(
        client_poll.clone(),
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

pub async fn report_heartbeat(
    client_poll: Arc<ClientPool>,
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

        time::sleep(Duration::from_millis(1000)).await;
    }
}
