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
use protocol::broker::broker_mqtt::{
    broker_mqtt_service_client::BrokerMqttServiceClient, DeleteSessionReply, DeleteSessionRequest,
    SendLastWillMessageReply, SendLastWillMessageRequest,
};
use tonic::transport::Channel;

pub mod call;

impl_retriable_request!(
    DeleteSessionRequest,
    BrokerMqttServiceClient<Channel>,
    DeleteSessionReply,
    delete_session,
    "MqttBrokerInnerService",
    "DeleteSession"
);

impl_retriable_request!(
    SendLastWillMessageRequest,
    BrokerMqttServiceClient<Channel>,
    SendLastWillMessageReply,
    send_last_will_message,
    "MqttBrokerInnerService",
    "SendLastWillMessage"
);
