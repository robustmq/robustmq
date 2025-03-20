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

use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::{connector::MQTTConnector, status::MQTTStatus};
use prost::Message;
use protocol::placement_center::placement_center_mqtt::CreateConnectorRequest;

use crate::{
    core::error::PlacementCenterError,
    mqtt::{
        cache::MqttCacheManager,
        controller::call_broker::{update_cache_by_add_connector, MQTTInnerCallManager},
    },
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
};

pub async fn update_connector_status_to_idle(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    mqtt_cache: &Arc<MqttCacheManager>,
    cluster_name: &str,
    connector_name: &str,
) -> Result<(), PlacementCenterError> {
    update_connector_status(
        raft_machine_apply,
        call_manager,
        client_pool,
        mqtt_cache,
        cluster_name,
        connector_name,
        MQTTStatus::Idle,
    )
    .await
}

pub async fn update_connector_status_to_running(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    mqtt_cache: &Arc<MqttCacheManager>,
    cluster_name: &str,
    connector_name: &str,
) -> Result<(), PlacementCenterError> {
    update_connector_status(
        raft_machine_apply,
        call_manager,
        client_pool,
        mqtt_cache,
        cluster_name,
        connector_name,
        MQTTStatus::Running,
    )
    .await
}

async fn update_connector_status(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    mqtt_cache: &Arc<MqttCacheManager>,
    cluster_name: &str,
    connector_name: &str,
    status: MQTTStatus,
) -> Result<(), PlacementCenterError> {
    if let Some(mut connector) = mqtt_cache.get_connector(cluster_name, connector_name) {
        connector.status = status;

        if connector.status == MQTTStatus::Idle {
            connector.broker_id = None;
        }

        let req = CreateConnectorRequest {
            cluster_name: connector.cluster_name.clone(),
            connector_name: connector.connector_name.clone(),
            connector: connector.encode(),
        };
        save_connector(raft_machine_apply, req, call_manager, client_pool).await?;
    }
    Ok(())
}

pub async fn save_connector(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: CreateConnectorRequest,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSetConnector,
        CreateConnectorRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let connector = serde_json::from_slice::<MQTTConnector>(&req.connector)?;
    update_cache_by_add_connector(&req.cluster_name, call_manager, client_pool, connector).await?;
    Ok(())
}
