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

use async_trait::async_trait;
use metadata_struct::connection::NetworkConnection;
use network_server::{command::Command, common::packet::ResponsePackage};
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;

pub struct KafkaCommand {}

impl KafkaCommand {
    pub fn new() -> Self {
        return KafkaCommand {};
    }
}

#[async_trait]
impl Command for KafkaCommand {
    async fn apply(
        &self,
        tcp_connection: NetworkConnection,
        addr: SocketAddr,
        robust_packet: RobustMQPacket,
    ) -> Option<ResponsePackage> {
        None
    }
}
