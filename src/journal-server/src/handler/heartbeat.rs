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
use std::time::Duration;

 use grpc_clients::placement::placement::call::{heartbeat, register_node, unregister_node};
 use grpc_clients::poll::ClientPool;
use common_base::config::journal_server::JournalServerConfig;
use common_base::tools::get_local_ip;
use log::{debug, error, info};
use protocol::placement_center::generate::common::ClusterType;
use protocol::placement_center::generate::placement::{
    HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest,
};
use tokio::time;

pub async fn register_storage_engine_node(
    client_poll: Arc<ClientPool>,
    config: JournalServerConfig,
) {
    let req = RegisterNodeRequest {
        cluster_type: ClusterType::JournalServer.into(),
        cluster_name: config.cluster_name,
        node_id: config.node_id,
        node_ip: get_local_ip(),
        extend_info: "".to_string(),
        ..Default::default()
    };
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
    let req = UnRegisterNodeRequest {
        cluster_type: ClusterType::JournalServer.into(),
        cluster_name: config.cluster_name,
        node_id: config.node_id,
    };

    match unregister_node(client_poll.clone(), config.placement_center, req.clone()).await {
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
        let req = HeartbeatRequest {
            cluster_name: config.cluster_name.clone(),
            cluster_type: ClusterType::JournalServer.into(),
            node_id: config.node_id,
        };
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
