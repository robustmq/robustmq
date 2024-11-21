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
use prost::Message;
use protocol::broker_mqtt::broker_mqtt_connection::{ListConnectionReply, ListConnectionRequest};

use crate::mqtt::{retry_call, MqttBrokerPlacementInterface, MqttBrokerService};
use crate::pool::ClientPool;

pub async fn mqtt_broker_list_connection(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListConnectionRequest,
) -> Result<ListConnectionReply, CommonError> {
    let request_date = ListConnectionRequest::encode_to_vec(&request);
    match retry_call(
        MqttBrokerService::Connection,
        MqttBrokerPlacementInterface::ListConnection,
        client_pool,
        addrs,
        request_date,
    )
    .await
    {
        Ok(data) => match ListConnectionReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
