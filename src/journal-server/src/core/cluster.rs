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

use common_base::error::common::CommonError;
use common_base::tools::{get_local_ip, now_second};
use common_config::broker::broker_config;
use common_config::broker::config::BrokerConfig;
use grpc_clients::placement::inner::call::{heartbeat, register_node, unregister_node};
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::node_extend::JournalNodeExtend;
use metadata_struct::placement::node::BrokerNode;
use protocol::placement_center::placement_center_inner::{
    ClusterType, HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest,
};
use tokio::{sync::broadcast, time::sleep};
use tracing::{debug, error};

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::tool::loop_select;

pub async fn register_journal_node(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
) -> Result<(), CommonError> {
    let conf = broker_config();
    let local_ip = get_local_ip();
    let extend = JournalNodeExtend {
        data_fold: conf.journal_storage.data_path.clone(),
        tcp_addr: format!("{}:{}", local_ip, conf.journal_server.tcp_port),
    };

    let node = BrokerNode {
        cluster_type: ClusterType::JournalServer.as_str_name().to_string(),
        cluster_name: conf.cluster_name.to_owned(),
        node_id: conf.broker_id,
        node_ip: local_ip.clone(),
        node_inner_addr: format!("{}:{}", local_ip.clone(), conf.grpc_port),
        extend: extend.encode(),
        start_time: cache_manager.get_start_time(),
        register_time: now_second(),
    };

    let req = RegisterNodeRequest {
        node: node.encode(),
    };
    register_node(client_pool, &conf.get_placement_center_addr(), req.clone()).await?;
    debug!("Node {} register successfully", conf.broker_id);
    Ok(())
}

pub async fn unregister_journal_node(
    client_pool: Arc<ClientPool>,
    config: BrokerConfig,
) -> Result<(), CommonError> {
    let req = UnRegisterNodeRequest {
        cluster_type: ClusterType::JournalServer.into(),
        cluster_name: config.cluster_name.clone(),
        node_id: config.broker_id,
    };
    unregister_node(
        &client_pool,
        &config.get_placement_center_addr(),
        req.clone(),
    )
    .await?;
    debug!("Node {} exits successfully", config.broker_id);
    Ok(())
}

pub fn report_heartbeat(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let client_pool_clone = client_pool.clone();
    let cache_manager_clone = cache_manager.clone();

    tokio::spawn(async move {
        sleep(Duration::from_secs(10)).await;

        let ac_fn = async || -> Result<(), JournalServerError> {
            report_report0(&client_pool_clone, &cache_manager_clone).await;
            Ok(())
        };

        loop_select(ac_fn, 1, &stop_send).await;
        debug!("Heartbeat reporting thread exited successfully");
    });
}

async fn report_report0(client_pool: &Arc<ClientPool>, cache_manager: &Arc<CacheManager>) {
    let config = broker_config();
    let req = HeartbeatRequest {
        cluster_name: config.cluster_name.clone(),
        cluster_type: ClusterType::JournalServer.into(),
        node_id: config.broker_id,
    };
    match heartbeat(
        client_pool,
        &config.get_placement_center_addr(),
        req.clone(),
    )
    .await
    {
        Ok(_) => {
            debug!(
                "Node {} successfully reports the heartbeat communication",
                config.broker_id
            );
        }
        Err(e) => {
            if e.to_string().contains("Node") && e.to_string().contains("does not exist") {
                if let Err(e) = register_journal_node(client_pool, cache_manager).await {
                    error!("{}", e);
                }
            } else {
                error!("{}", e);
            }
        }
    }
    sleep(Duration::from_secs(1)).await;
}

pub fn report_monitor(client_pool: Arc<ClientPool>, stop_send: broadcast::Sender<bool>) {
    tokio::spawn(async move {
        let ac_fn = async || -> Result<(), JournalServerError> {
            report_monitor0(client_pool.clone()).await;
            Ok(())
        };

        loop_select(ac_fn, 1, &stop_send).await;
        debug!("Monitor reporting thread exited successfully");
    });
}

async fn report_monitor0(_client_pool: Arc<ClientPool>) {
    sleep(Duration::from_secs(1)).await;
}
