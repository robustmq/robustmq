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

use crate::macros::impl_retriable_request;
use protocol::broker::broker::{
    broker_service_client::BrokerServiceClient, GetQosDataByClientIdReply,
    GetQosDataByClientIdRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};
use tonic::transport::Channel;

pub mod call;

impl_retriable_request!(
    UpdateCacheRequest,
    BrokerServiceClient<Channel>,
    UpdateCacheReply,
    update_cache,
    "BrokerService",
    "UpdateCache"
);

impl_retriable_request!(
    SendLastWillMessageRequest,
    BrokerServiceClient<Channel>,
    SendLastWillMessageReply,
    send_last_will_message,
    "BrokerService",
    "SendLastWillMessage"
);

impl_retriable_request!(
    GetQosDataByClientIdRequest,
    BrokerServiceClient<Channel>,
    GetQosDataByClientIdReply,
    get_qos_data_by_client_id,
    "BrokerService",
    "GetQosDataByClientId"
);
