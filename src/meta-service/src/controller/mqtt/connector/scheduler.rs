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

use super::status::{save_connector, update_connector_status_to_running};
use crate::{
    controller::mqtt::{
        call_broker::MQTTInnerCallManager, connector::status::update_connector_status_to_idle,
    },
    core::{cache::CacheManager, error::MetaServiceError},
    raft::route::apply::StorageDriver,
};
use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_second},
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::status::MQTTStatus;
use protocol::meta::meta_service_mqtt::CreateConnectorRequest;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::{info, warn};

pub async fn start_connector_scheduler(
    cache_manager: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        scheduler_thread(raft_machine_apply, call_manager, client_pool, cache_manager).await;
        Ok(())
    };

    loop_select_ticket(ac_fn, 1, &stop_send).await;
}

async fn scheduler_thread(
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
) {
    if let Err(e) =
        check_heartbeat(raft_machine_apply, call_manager, client_pool, cache_manager).await
    {
        info!("check heartbeat error: {:?}", e);
    }

    if let Err(e) =
        start_stop_connector_thread(raft_machine_apply, call_manager, client_pool, cache_manager)
            .await
    {
        info!("start stop connector thread error: {:?}", e);
    }
}

async fn check_heartbeat(
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
) -> Result<(), MetaServiceError> {
    let config = broker_config();
    for heartbeat in cache_manager.get_all_connector_heartbeat() {
        let connector = if let Some(connector) =
            cache_manager.get_connector(&heartbeat.cluster_name, &heartbeat.connector_name)
        {
            connector
        } else {
            cache_manager
                .remove_connector_heartbeat(&heartbeat.cluster_name, &heartbeat.connector_name);
            continue;
        };

        if now_second() - heartbeat.last_heartbeat > config.meta_runtime.heartbeat_timeout_ms / 1000
        {
            info!(
                "cluster:{},Connector {} heartbeat expired, rescheduled, new node: {}",
                connector.cluster_name, connector.connector_name, 1
            );

            update_connector_status_to_idle(
                raft_machine_apply,
                call_manager,
                client_pool,
                cache_manager,
                &heartbeat.cluster_name,
                &heartbeat.connector_name,
            )
            .await?
        }
    }
    Ok(())
}

async fn start_stop_connector_thread(
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
) -> Result<(), MetaServiceError> {
    for mut connector in cache_manager.get_all_connector() {
        if connector.broker_id.is_none() && connector.status == MQTTStatus::Running {
            warn!("Connector {} has an abnormal state, which is Running, but the execution node is empty.", connector.cluster_name);
        }

        if connector.broker_id.is_none() {
            connector.broker_id =
                Some(calc_connector_broker(cache_manager, &connector.cluster_name).await?);
            connector.status = MQTTStatus::Idle;

            info!("Connector execution nodes are assigned and Connector {} is assigned to Broker {:?} for execution.",
                connector.connector_name,
                connector.broker_id
            );

            let req = CreateConnectorRequest {
                cluster_name: connector.cluster_name.clone(),
                connector_name: connector.connector_name.clone(),
                connector: connector.encode(),
            };

            save_connector(raft_machine_apply, req, call_manager, client_pool).await?;
            continue;
        }

        if connector.status == MQTTStatus::Running {
            continue;
        }

        if connector.status == MQTTStatus::Idle {
            info!(
                "Connector {} state changes from Idle to Running",
                connector.connector_name
            );

            update_connector_status_to_running(
                raft_machine_apply,
                call_manager,
                client_pool,
                cache_manager,
                &connector.cluster_name,
                &connector.connector_name,
            )
            .await?;
        }
    }
    Ok(())
}

async fn calc_connector_broker(
    cache_manager: &Arc<CacheManager>,
    cluster_name: &str,
) -> Result<u64, MetaServiceError> {
    let mut connector_broker_id_nums = HashMap::new();
    for connector in cache_manager.get_all_connector() {
        if let Some(broker_id) = connector.broker_id {
            if let Some(num) = connector_broker_id_nums.get(&broker_id) {
                connector_broker_id_nums.insert(broker_id, *num + 1);
            } else {
                connector_broker_id_nums.insert(broker_id, 1);
            }
        }
    }

    let mut all_broker_id_nums = HashMap::new();
    for broker_id in cache_manager.get_broker_node_id_by_cluster(cluster_name) {
        if let Some(num) = connector_broker_id_nums.get(&broker_id) {
            all_broker_id_nums.insert(broker_id, *num);
        } else {
            all_broker_id_nums.insert(broker_id, 0);
        }
    }

    let mut broker_id_num = 0;
    let mut broker_id = -1;
    for (id, num) in all_broker_id_nums {
        if broker_id_num > num {
            broker_id_num = num;
            broker_id = id as i64
        }
    }

    if broker_id == -1 {
        return Err(MetaServiceError::NoAvailableBrokerNode);
    }

    Ok(broker_id as u64)
}
