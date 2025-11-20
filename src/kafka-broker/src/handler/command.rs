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

use axum::async_trait;
use kafka_protocol::messages::api_versions_response::ApiVersion;
use kafka_protocol::messages::{ApiKey, ApiVersionsResponse};
use metadata_struct::connection::NetworkConnection;
use network_server::{command::Command, common::packet::ResponsePackage};
use protocol::kafka::packet::KafkaPacket;
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct KafkaCommand {}

impl KafkaCommand {
    fn handle_api_versions(tcp_connection: &NetworkConnection) -> ResponsePackage {
        let api_versions: Vec<ApiVersion> = ApiKey::iter()
            .map(|k| {
                let range = ApiKey::valid_versions(&k);
                ApiVersion::default()
                    .with_api_key(k as i16)
                    .with_min_version(range.min)
                    .with_max_version(range.max)
            })
            .collect();
        let res = ApiVersionsResponse::default()
            .with_error_code(0)
            .with_api_keys(api_versions)
            .with_throttle_time_ms(0);
        ResponsePackage::build(
            tcp_connection.connection_id,
            RobustMQPacket::KAFKA(KafkaPacket::ApiVersionResponse(res)),
        )
    }
}

#[async_trait]
impl Command for KafkaCommand {
    async fn apply(
        &self,
        tcp_connection: &NetworkConnection,
        _addr: &SocketAddr,
        robust_packet: &RobustMQPacket,
    ) -> Option<ResponsePackage> {
        let packet = robust_packet.get_kafka_packet().unwrap();
        match packet.clone() {
            KafkaPacket::ApiVersionReq(_) => Some(Self::handle_api_versions(tcp_connection)),
            KafkaPacket::ProduceReq(_) => None,
            KafkaPacket::FetchReq(_) => None,
            _ => None,
        }
    }
}

pub fn create_command() -> Arc<Box<dyn Command + Send + Sync>> {
    let storage: Box<dyn Command + Send + Sync> = Box::new(KafkaCommand::default());
    Arc::new(storage)
}
