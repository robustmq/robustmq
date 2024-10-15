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

use common_base::error::common::CommonError;
use mobc::Connection;
use prost::Message;
use protocol::broker_mqtt::broker_mqtt_admin::{ClusterStatusReply, ClusterStatusRequest};

use super::MqttBrokerAdminServiceManager;

pub(crate) async fn inner_cluster_status(
    mut client: Connection<MqttBrokerAdminServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match ClusterStatusRequest::decode(request.as_ref()) {
        Ok(request) => match client.cluster_status(request).await {
            Ok(result) => Ok(ClusterStatusReply::encode_to_vec(&result.into_inner())),
            Err(e) => Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => Err(CommonError::CommmonError(e.to_string())),
    }
}
