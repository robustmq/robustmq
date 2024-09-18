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

use clients::{
    placement::placement::call::{heartbeat, register_node, un_register_node},
    poll::ClientPool,
};
use common_base::{config::journal_server::JournalServerConfig, tools::get_local_ip};
use log::{debug, error, info};
use protocol::placement_center::generate::{
    common::ClusterType,
    placement::{HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest},
};
use std::{sync::Arc, time::Duration};
use tokio::time;

pub async fn register_storage_engine_node(
    client_poll: Arc<ClientPool>,
    config: JournalServerConfig,
) {
    let mut req = RegisterNodeRequest::default();
    req.cluster_type = ClusterType::JournalServer.into();
    req.cluster_name = config.cluster_name;
    req.node_id = config.node_id;
    req.node_ip = get_local_ip();
    req.extend_info = "".to_string();
    match register_node(client_poll.clone(), config.placement_center, req.clone()).await {
        Ok(_) => {
            info!("Node {} has been successfully registered", config.node_id);
        }
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }
}

pub async fn unregister_storage_engine_node(
    client_poll: Arc<ClientPool>,
    config: JournalServerConfig,
) {
    let mut req = UnRegisterNodeRequest::default();
    req.cluster_type = ClusterType::JournalServer.into();
    req.cluster_name = config.cluster_name;
    req.node_id = config.node_id;

    match un_register_node(client_poll.clone(), config.placement_center, req.clone()).await {
        Ok(_) => {
            info!("Node {} exits successfully", config.node_id);
        }
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }
}

pub async fn report_heartbeat(client_poll: Arc<ClientPool>, config: JournalServerConfig) {
    loop {
        let mut req = HeartbeatRequest::default();
        req.cluster_name = config.cluster_name.clone();
        req.cluster_type = ClusterType::JournalServer.into();
        req.node_id = config.node_id;
        match heartbeat(
            client_poll.clone(),
            config.placement_center.clone(),
            req.clone(),
        )
        .await
        {
            Ok(_) => {
                debug!(
                    "Node {} successfully reports the heartbeat communication",
                    config.node_id
                );
                break;
            }
            Err(e) => {
                error!("{}", e);
            }
        }
        time::sleep(Duration::from_millis(1000)).await;
    }
}
