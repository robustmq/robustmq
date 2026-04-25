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
    CreateMailReply, CreateMailRequest, DeleteMailReply, DeleteMailRequest, ListMailReply,
    ListMailRequest,
};
use tonic::transport::Channel;
use tonic::Streaming;

use crate::macros::impl_retriable_request;

pub mod call;

impl_retriable_request!(
    CreateMailRequest,
    Mq9ServiceClient<Channel>,
    CreateMailReply,
    create_mail,
    "Mq9Service",
    "CreateMail",
    true
);

impl_retriable_request!(
    DeleteMailRequest,
    Mq9ServiceClient<Channel>,
    DeleteMailReply,
    delete_mail,
    "Mq9Service",
    "DeleteMail",
    true
);

impl_retriable_request!(
    ListMailRequest,
    Mq9ServiceClient<Channel>,
    Streaming<ListMailReply>,
    list_mail,
    "Mq9Service",
    "ListMail",
    true
);
