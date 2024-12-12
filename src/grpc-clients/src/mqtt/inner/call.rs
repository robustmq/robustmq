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
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};

use crate::pool::ClientPool;
use crate::utils::retry_call;

pub async fn broker_mqtt_delete_session(
    client_pool: &ClientPool,
    addrs: &[std::net::SocketAddr],
    request: DeleteSessionRequest,
) -> Result<DeleteSessionReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn broker_mqtt_update_cache(
    client_pool: &ClientPool,
    addrs: &[std::net::SocketAddr],
    request: UpdateCacheRequest,
) -> Result<UpdateCacheReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn send_last_will_message(
    client_pool: &ClientPool,
    addrs: &[std::net::SocketAddr],
    request: SendLastWillMessageRequest,
) -> Result<SendLastWillMessageReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}
