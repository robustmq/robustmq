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
use protocol::broker::broker_mqtt::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateMqttCacheReply, UpdateMqttCacheRequest,
};

use crate::pool::ClientPool;

macro_rules! generate_mqtt_inner_service_call {
    ($fn_name:ident, $req_ty:ty, $rep_ty:ty, $variant:ident) => {
        pub async fn $fn_name(
            client_pool: &ClientPool,
            addrs: &[impl AsRef<str>],
            request: $req_ty,
        ) -> Result<$rep_ty, CommonError> {
            $crate::utils::retry_call(client_pool, addrs, request).await
        }
    };
}

generate_mqtt_inner_service_call!(
    broker_mqtt_delete_session,
    DeleteSessionRequest,
    DeleteSessionReply,
    DeleteSession
);

generate_mqtt_inner_service_call!(
    broker_mqtt_update_cache,
    UpdateMqttCacheRequest,
    UpdateMqttCacheReply,
    UpdateMqttCache
);

generate_mqtt_inner_service_call!(
    send_last_will_message,
    SendLastWillMessageRequest,
    SendLastWillMessageReply,
    SendLastWillMessage
);
