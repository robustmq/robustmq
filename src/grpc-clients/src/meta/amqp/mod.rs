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

use protocol::meta::meta_service_amqp::amqp_service_client::AmqpServiceClient;
use protocol::meta::meta_service_amqp::{
    DeleteBindingReply, DeleteBindingRequest, DeleteExchangeReply, DeleteExchangeRequest,
    DeleteQueueReply, DeleteQueueRequest, ListBindingReply, ListBindingRequest, ListExchangeReply,
    ListExchangeRequest, ListQueueReply, ListQueueRequest, SetBindingReply, SetBindingRequest,
    SetExchangeReply, SetExchangeRequest, SetQueueReply, SetQueueRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

impl_retriable_request!(
    SetExchangeRequest,
    AmqpServiceClient<Channel>,
    SetExchangeReply,
    set_exchange,
    "AmqpService",
    "SetExchange",
    true
);

impl_retriable_request!(
    DeleteExchangeRequest,
    AmqpServiceClient<Channel>,
    DeleteExchangeReply,
    delete_exchange,
    "AmqpService",
    "DeleteExchange",
    true
);

impl_retriable_request!(
    ListExchangeRequest,
    AmqpServiceClient<Channel>,
    ListExchangeReply,
    list_exchange,
    "AmqpService",
    "ListExchange",
    true
);

impl_retriable_request!(
    SetQueueRequest,
    AmqpServiceClient<Channel>,
    SetQueueReply,
    set_queue,
    "AmqpService",
    "SetQueue",
    true
);

impl_retriable_request!(
    DeleteQueueRequest,
    AmqpServiceClient<Channel>,
    DeleteQueueReply,
    delete_queue,
    "AmqpService",
    "DeleteQueue",
    true
);

impl_retriable_request!(
    ListQueueRequest,
    AmqpServiceClient<Channel>,
    ListQueueReply,
    list_queue,
    "AmqpService",
    "ListQueue",
    true
);

impl_retriable_request!(
    SetBindingRequest,
    AmqpServiceClient<Channel>,
    SetBindingReply,
    set_binding,
    "AmqpService",
    "SetBinding",
    true
);

impl_retriable_request!(
    DeleteBindingRequest,
    AmqpServiceClient<Channel>,
    DeleteBindingReply,
    delete_binding,
    "AmqpService",
    "DeleteBinding",
    true
);

impl_retriable_request!(
    ListBindingRequest,
    AmqpServiceClient<Channel>,
    ListBindingReply,
    list_binding,
    "AmqpService",
    "ListBinding",
    true
);
