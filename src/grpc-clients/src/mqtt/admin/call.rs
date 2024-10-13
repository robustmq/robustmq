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
use protocol::broker_server::generate::admin::{ClusterStatusReply, ClusterStatusRequest};

use crate::mqtt::{retry_call, MQTTBrokerPlacementInterface, MQTTBrokerService};
use crate::poll::ClientPool;

pub async fn cluster_status(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ClusterStatusRequest,
) -> Result<ClusterStatusReply, CommonError> {
    let request_data = ClusterStatusRequest::encode_to_vec(&request);
    match retry_call(
        MQTTBrokerService::Admin,
        MQTTBrokerPlacementInterface::ClusterStatus,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ClusterStatusReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
