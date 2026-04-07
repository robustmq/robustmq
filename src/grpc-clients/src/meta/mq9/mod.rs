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

use protocol::meta::meta_service_mq9::mq9_service_client::Mq9ServiceClient;
use protocol::meta::meta_service_mq9::{
    CreateEmailReply, CreateEmailRequest, DeleteEmailReply, DeleteEmailRequest, ListEmailReply,
    ListEmailRequest,
};
use tonic::transport::Channel;
use tonic::Streaming;

use crate::macros::impl_retriable_request;

pub mod call;

impl_retriable_request!(
    CreateEmailRequest,
    Mq9ServiceClient<Channel>,
    CreateEmailReply,
    create_email,
    "Mq9Service",
    "CreateEmail",
    true
);

impl_retriable_request!(
    DeleteEmailRequest,
    Mq9ServiceClient<Channel>,
    DeleteEmailReply,
    delete_email,
    "Mq9Service",
    "DeleteEmail",
    true
);

impl_retriable_request!(
    ListEmailRequest,
    Mq9ServiceClient<Channel>,
    Streaming<ListEmailReply>,
    list_email,
    "Mq9Service",
    "ListEmail",
    true
);
