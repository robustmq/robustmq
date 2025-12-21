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

use crate::core::cache::StorageCacheManager;
use crate::core::error::get_journal_server_code;
use crate::handler::data::{read_data_req, write_data_req};
use crate::segment::write::WriteManager;
use crate::segment::SegmentIdentity;
use axum::async_trait;
use common_config::broker::broker_config;
use metadata_struct::connection::NetworkConnection;
use network_server::command::Command;
use network_server::common::packet::ResponsePackage;
use protocol::storage::codec::StorageEnginePacket;
use protocol::storage::protocol::{
    ApiKey, ReadRespBody, RespHeader, StorageEngineNetworkError, WriteRespBody,
};
use protocol::{robust::RobustMQPacket, storage::protocol::WriteResp};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error};

/// a dispatcher struct to handle all commands from journal clients
#[derive(Clone)]
pub struct StorageEngineHandlerCommand {
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    write_manager: Arc<WriteManager>,
}

impl StorageEngineHandlerCommand {
    pub fn new(
        cache_manager: Arc<StorageCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        write_manager: Arc<WriteManager>,
    ) -> Self {
        StorageEngineHandlerCommand {
            cache_manager,
            rocksdb_engine_handler,
            write_manager,
        }
    }
}

#[async_trait]
impl Command for StorageEngineHandlerCommand {
    async fn apply(
        &self,
        tcp_connection: &NetworkConnection,
        _addr: &SocketAddr,
        packet: &RobustMQPacket,
    ) -> Option<ResponsePackage> {
        debug!("recv packet: {:?}", packet);
        let pack = match packet {
            RobustMQPacket::StorageEngine(pack) => pack.clone(),
            _ => {
                return None;
            }
        };

        match pack {
            StorageEnginePacket::WriteReq(request) => {
                let segment_iden = SegmentIdentity {
                    shard_name: request.body.shard_name,
                    segment: request.body.segment,
                };
                let messages = request.body.messages;

                let (resp_body, error) = match write_data_req(
                    &self.cache_manager,
                    &self.write_manager,
                    &segment_iden,
                    &messages,
                )
                .await
                {
                    Ok(status) => (WriteRespBody { status }, None),
                    Err(e) => (
                        WriteRespBody::default(),
                        Some(StorageEngineNetworkError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        }),
                    ),
                };

                let resp = WriteResp {
                    header: RespHeader {
                        api_key: ApiKey::Write.into(),
                        error,
                    },
                    body: resp_body,
                };

                let response = ResponsePackage::build(
                    tcp_connection.connection_id,
                    RobustMQPacket::StorageEngine(StorageEnginePacket::WriteResp(resp)),
                );
                return Some(response);
            }

            StorageEnginePacket::ReadReq(request) => {
                let req_body = request.body;
                let config = broker_config();
                let (resp_body, error) = match read_data_req(
                    &self.cache_manager,
                    &self.rocksdb_engine_handler,
                    &req_body,
                    config.broker_id,
                )
                .await
                {
                    Ok(messages) => (ReadRespBody { messages }, None),
                    Err(e) => (
                        ReadRespBody::default(),
                        Some(StorageEngineNetworkError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        }),
                    ),
                };

                let resp = protocol::storage::protocol::ReadResp {
                    header: RespHeader {
                        api_key: ApiKey::Read.into(),
                        error,
                    },
                    body: resp_body,
                };

                let response = ResponsePackage::build(
                    tcp_connection.connection_id,
                    RobustMQPacket::StorageEngine(StorageEnginePacket::ReadResp(resp)),
                );
                return Some(response);
            }

            _ => {
                error!(
                    "storage engine server received an unrecognized request, request info: {:?}",
                    packet
                );
            }
        }
        None
    }
}
