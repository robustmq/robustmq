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
use crate::core::handler::DataHandler;
use crate::segment::manager::SegmentFileManager;
use crate::server::connection::NetworkConnection;
use crate::server::connection_manager::ConnectionManager;
use grpc_clients::pool::ClientPool;
use protocol::storage::codec::JournalEnginePacket;
use protocol::storage::journal_engine::{
    ApiKey, ApiVersion, JournalEngineError, ReadResp, ReadRespBody, RespHeader, WriteResp,
    WriteRespBody,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error};

/// a dispatcher struct to handle all commands from journal clients
#[derive(Clone)]
pub struct Command {
    data_handler: DataHandler,
}

impl Command {
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
        Command { data_handler }
    }

    pub async fn apply(
        &self,
        _connect_manager: Arc<ConnectionManager>,
        _tcp_connection: NetworkConnection,
        _addr: SocketAddr,
        packet: JournalEnginePacket,
    ) -> Option<JournalEnginePacket> {
        debug!("recv packet: {:?}", packet);
        match packet {
            /* Data Handler */
            JournalEnginePacket::WriteReq(request) => {
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
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(WriteRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::WriteResp(resp));
            }

            JournalEnginePacket::ReadReq(request) => {
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
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(ReadRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::ReadResp(resp));
            }

            _ => {
                error!(
                    "server received an unrecognized request, request info: {:?}",
                    packet
                );
            }
        }
        None
    }
}
