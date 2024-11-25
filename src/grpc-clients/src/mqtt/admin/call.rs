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

use common_base::error::common::CommonError;
use prost::Message as _;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply,
    DeleteUserRequest, ListConnectionReply, ListConnectionRequest, ListUserReply, ListUserRequest,
};

use crate::mqtt::{retry_call, MqttBrokerPlacementInterface, MqttBrokerPlacementReply, MqttBrokerPlacementRequest, MqttBrokerService};
use crate::pool::ClientPool;

// ---- cluster ------
pub async fn cluster_status(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ClusterStatusRequest,
) -> Result<ClusterStatusReply, CommonError> {
    let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::ClusterStatus(request)).await?;
    match reply {
        MqttBrokerPlacementReply::ClusterStatus(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

// ------ user -------
pub async fn mqtt_broker_list_user(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListUserRequest,
) -> Result<ListUserReply, CommonError> {
    let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::ListUser(request)).await?;
    match reply {
        MqttBrokerPlacementReply::ListUser(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_create_user(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateUserRequest,
) -> Result<CreateUserReply, CommonError> {
    let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::CreateUser(request)).await?;
    match reply {
        MqttBrokerPlacementReply::CreateUser(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_delete_user(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteUserRequest,
) -> Result<DeleteUserReply, CommonError> {
    let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::DeleteUser(request)).await?;
    match reply {
        MqttBrokerPlacementReply::DeleteUser(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

// ------- connection  -----------
pub async fn mqtt_broker_list_connection(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListConnectionRequest,
) -> Result<ListConnectionReply, CommonError> {
    let request_date = ListConnectionRequest::encode_to_vec(&request);
    match retry_call(
        MqttBrokerService::Admin,
        MqttBrokerPlacementInterface::ListConnection,
        client_pool,
        addrs,
        request_date,
    )
    .await
    {
        Ok(data) => match ListConnectionReply::decode(data.as_ref()) {
            Ok(data) => Ok(data),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
