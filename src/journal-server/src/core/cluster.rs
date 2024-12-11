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

use common_base::config::journal_server::{journal_server_conf, JournalServerConfig};
use common_base::error::common::CommonError;
use common_base::tools::get_local_ip;
use grpc_clients::placement::inner::call::{heartbeat, register_node, unregister_node};
use grpc_clients::pool::ClientPool;
use log::{debug, error, info};
use metadata_struct::journal::node_extend::JournalNodeExtend;
use protocol::placement_center::placement_center_inner::{
    ClusterType, HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest,
};
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

#[derive(Clone, Default)]
pub struct JournalEngineClusterConfig {
    pub enable_auto_create_shard: bool,
    pub last_update_local_cache_time: u64,
}

pub async fn register_journal_node(
    client_pool: Arc<ClientPool>,
    config: JournalServerConfig,
) -> Result<(), CommonError> {
    let conf = journal_server_conf();
    let extend = JournalNodeExtend {
        data_fold: conf.storage.data_path.clone(),
        tcp_addr: format!("{}:{}", get_local_ip(), conf.network.tcp_port),
        tcps_addr: format!("{}:{}", get_local_ip(), conf.network.tcps_port),
    };

    let req = RegisterNodeRequest {
        cluster_type: ClusterType::JournalServer.into(),
        cluster_name: config.cluster_name,
        node_id: config.node_id,
        node_ip: get_local_ip(),
        node_inner_addr: format!("{}:{}", get_local_ip(), conf.network.grpc_port),
        extend_info: serde_json::to_string(&extend)?,
    };
    register_node(client_pool.clone(), &config.placement_center, req.clone()).await?;
    info!("Node {} register successfully", config.node_id);
    Ok(())
}

pub async fn unregister_journal_node(
    client_pool: Arc<ClientPool>,
    config: JournalServerConfig,
) -> Result<(), CommonError> {
    let req = UnRegisterNodeRequest {
        cluster_type: ClusterType::JournalServer.into(),
        cluster_name: config.cluster_name,
        node_id: config.node_id,
    };
    unregister_node(client_pool.clone(), &config.placement_center, req.clone()).await?;
    info!("Node {} exits successfully", config.node_id);
    Ok(())
}

pub async fn report_heartbeat(client_pool: Arc<ClientPool>, stop_send: broadcast::Sender<bool>) {
    loop {
        let mut stop_recv = stop_send.subscribe();
        select! {
            val = stop_recv.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        debug!("{}","Heartbeat reporting thread exited successfully");
                        break;
                    }
                }
            }
            _ = report(client_pool.clone()) => {

            }
        }
    }
}

async fn report(client_pool: Arc<ClientPool>) {
    let config = journal_server_conf();
    let req = HeartbeatRequest {
        cluster_name: config.cluster_name.clone(),
        cluster_type: ClusterType::JournalServer.into(),
        node_id: config.node_id,
    };
    match heartbeat(client_pool.clone(), &config.placement_center, req.clone()).await {
        Ok(_) => {
            debug!(
                "Node {} successfully reports the heartbeat communication",
                config.node_id
            );
        }
        Err(e) => {
            if e.to_string().contains("Node") && e.to_string().contains("does not exist") {
                if let Err(e) = register_journal_node(client_pool.clone(), config.clone()).await {
                    error!("{}", e);
                }
            }
            error!("{}", e);
        }
    }
    sleep(Duration::from_secs(1)).await;
}
