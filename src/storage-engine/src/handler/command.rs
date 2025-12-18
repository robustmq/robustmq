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
use crate::handler::data::DataHandler;
use crate::segment::storage::manager::SegmentFileManager;
use axum::async_trait;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnection;
use network_server::command::Command;
use network_server::common::packet::ResponsePackage;
use protocol::robust::RobustMQPacket;
use protocol::storage::codec::StorageEnginePacket;
use protocol::storage::storage_engine_engine::{
    ApiKey, ApiVersion, ReadResp, ReadRespBody, RespHeader, StorageEngineError, WriteResp,
    WriteRespBody,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error};

/// a dispatcher struct to handle all commands from journal clients
#[derive(Clone)]
pub struct StorageEngineHandlerCommand {
    data_handler: DataHandler,
}

impl StorageEngineHandlerCommand {
    pub fn new(
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<StorageCacheManager>,
        segment_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        let data_handler = DataHandler::new(
            cache_manager,
            segment_file_manager,
            rocksdb_engine_handler,
            client_pool,
        );
        StorageEngineHandlerCommand { data_handler }
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
            /* Data Handler */
            StorageEnginePacket::WriteReq(request) => {
                let mut resp = WriteResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::Write.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.data_handler.write(request).await {
                    Ok(status) => {
                        resp.body = Some(WriteRespBody { status });
                    }
                    Err(e) => {
                        header.error = Some(StorageEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(WriteRespBody::default());
                    }
                }
                resp.header = Some(header);
                let response = ResponsePackage::build(
                    tcp_connection.connection_id,
                    RobustMQPacket::StorageEngine(StorageEnginePacket::WriteResp(resp)),
                );
                return Some(response);
            }

            StorageEnginePacket::ReadReq(request) => {
                let mut resp = ReadResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::Read.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.data_handler.read(request).await {
                    Ok(messages) => {
                        resp.body = Some(ReadRespBody { messages });
                    }
                    Err(e) => {
                        header.error = Some(StorageEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(ReadRespBody::default());
                    }
                }
                resp.header = Some(header);
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
