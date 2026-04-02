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

use protocol::meta::meta_service_nats::nats_service_client::NatsServiceClient;
use protocol::meta::meta_service_nats::{
    CreateNatsSubjectReply, CreateNatsSubjectRequest, DeleteNatsSubjectReply,
    DeleteNatsSubjectRequest, ListNatsSubjectReply, ListNatsSubjectRequest,
};
use tonic::transport::Channel;
use tonic::Streaming;

use crate::macros::impl_retriable_request;

pub mod call;

impl_retriable_request!(
    ListNatsSubjectRequest,
    NatsServiceClient<Channel>,
    Streaming<ListNatsSubjectReply>,
    list_nats_subject,
    "NatsService",
    "ListNatsSubject",
    true
);

impl_retriable_request!(
    CreateNatsSubjectRequest,
    NatsServiceClient<Channel>,
    CreateNatsSubjectReply,
    create_nats_subject,
    "NatsService",
    "CreateNatsSubject",
    true
);

impl_retriable_request!(
    DeleteNatsSubjectRequest,
    NatsServiceClient<Channel>,
    DeleteNatsSubjectReply,
    delete_nats_subject,
    "NatsService",
    "DeleteNatsSubject",
    true
);
