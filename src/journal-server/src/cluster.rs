use clients::{
    placement::placement::call::{heartbeat, register_node, un_register_node},
    ClientPool,
};
use common_base::{
    config::journal_server::JournalServerConfig,
    log::{debug_eninge, error_engine, info_meta},
    tools::get_local_ip,
};
use protocol::placement_center::generate::{
    common::ClusterType,
    placement::{HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest},
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

pub async fn register_storage_engine_node(
    client_poll: Arc<Mutex<ClientPool>>,
    config: JournalServerConfig,
) {
    let mut req = RegisterNodeRequest::default();
    req.cluster_type = ClusterType::StorageEngine.into();
    req.cluster_name = config.cluster_name;
    req.node_id = config.node_id;
    req.node_ip = get_local_ip();
    req.node_port = config.grpc_port;
    req.extend_info = "".to_string();
    match register_node(client_poll.clone(), config.placement_center, req.clone()).await {
        Ok(_) => {
            info_meta(&format!(
                "Node {} has been successfully registered",
                config.node_id
            ));
        }
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }
}

pub async fn unregister_storage_engine_node(
    client_poll: Arc<Mutex<ClientPool>>,
    config: JournalServerConfig,
) {
    let mut req = UnRegisterNodeRequest::default();
    req.cluster_type = ClusterType::StorageEngine.into();
    req.cluster_name = config.cluster_name;
    req.node_id = config.node_id;

    match un_register_node(client_poll.clone(), config.placement_center, req.clone()).await {
        Ok(_) => {
            info_meta(&format!("Node {} exits successfully", config.node_id));
        }
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }
}

pub async fn report_heartbeat(client_poll: Arc<Mutex<ClientPool>>, config: JournalServerConfig) {
    loop {
        let mut req = HeartbeatRequest::default();
        req.cluster_name = config.cluster_name.clone();
        req.cluster_type = ClusterType::StorageEngine.into();
        req.node_id = config.node_id;
        match heartbeat(
            client_poll.clone(),
            config.placement_center.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {
                debug_eninge(format!(
                    "Node {} successfully reports the heartbeat communication",
                    config.node_id
                ));
                break;
            }
            Err(e) => {
                error_engine(e.to_string());
            }
        }
        time::sleep(Duration::from_millis(1000)).await;
    }
}
