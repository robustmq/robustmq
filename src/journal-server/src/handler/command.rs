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

use std::net::SocketAddr;
use std::sync::Arc;

use grpc_clients::pool::ClientPool;
use log::{debug, error, info};
use protocol::journal_server::codec::JournalEnginePacket;
use protocol::journal_server::journal_engine::{
    ApiKey, ApiVersion, CreateShardResp, CreateShardRespBody, DeleteShardResp, DeleteShardRespBody,
    FetchOffsetResp, FetchOffsetRespBody, GetClusterMetadataResp, GetClusterMetadataRespBody,
    GetShardMetadataResp, GetShardMetadataRespBody, JournalEngineError, ListShardResp,
    ListShardRespBody, ReadResp, ReadRespBody, RespHeader, WriteResp, WriteRespBody,
};
use rocksdb_engine::RocksDBEngine;

use super::cluster::ClusterHandler;
use super::data::DataHandler;
use super::shard::ShardHandler;
use crate::core::cache::CacheManager;
use crate::core::error::get_journal_server_code;
use crate::segment::manager::SegmentFileManager;
use crate::server::connection::NetworkConnection;
use crate::server::connection_manager::ConnectionManager;

/// a dispatcher struct to handle all commands from journal clients
#[derive(Clone)]
pub struct Command {
    cluster_handler: ClusterHandler,
    shard_handler: ShardHandler,
    data_handler: DataHandler,
}

impl Command {
    pub fn new(
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
        segment_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        let cluster_handler = ClusterHandler::new(cache_manager.clone());
        let shard_handler = ShardHandler::new(cache_manager.clone(), client_pool.clone());
        let data_handler = DataHandler::new(
            cache_manager,
            segment_file_manager,
            rocksdb_engine_handler,
            client_pool,
        );
        Command {
            cluster_handler,
            shard_handler,
            data_handler,
        }
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
            /* Cluster Handler */
            JournalEnginePacket::GetClusterMetadataReq(_) => {
                let mut header = RespHeader {
                    api_key: ApiKey::GetClusterMetadata.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };

                let mut resp = GetClusterMetadataResp::default();
                match self.cluster_handler.get_cluster_metadata() {
                    Ok(data) => resp.body = Some(GetClusterMetadataRespBody { nodes: data }),
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(GetClusterMetadataRespBody::default());
                    }
                }

                resp.header = Some(header);
                return Some(JournalEnginePacket::GetClusterMetadataResp(resp));
            }

            /* Shard Handler */
            JournalEnginePacket::CreateShardReq(request) => {
                info!("recv create shard request: {:?}", request);
                let mut resp = CreateShardResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::CreateShard.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.shard_handler.create_shard(request).await {
                    Ok(()) => {
                        resp.body = Some(CreateShardRespBody {});
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(CreateShardRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::CreateShardResp(resp));
            }

            JournalEnginePacket::DeleteShardReq(request) => {
                info!("recv delete shard request: {:?}", request);
                let mut resp = DeleteShardResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::DeleteShard.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.shard_handler.delete_shard(request).await {
                    Ok(()) => {
                        resp.body = Some(DeleteShardRespBody {});
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(DeleteShardRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::DeleteShardResp(resp));
            }

            JournalEnginePacket::ListShardReq(request) => {
                info!("recv list shard request: {:?}", request);
                let mut resp = ListShardResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::ListShard.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.shard_handler.list_shard(request).await {
                    Ok(shards) => resp.body = Some(ListShardRespBody { shards }),
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(ListShardRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::ListShardResp(resp));
            }

            JournalEnginePacket::GetShardMetadataReq(request) => {
                let mut resp = GetShardMetadataResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::GetShardMetadata.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.shard_handler.get_shard_metadata(request).await {
                    Ok(shards) => {
                        resp.body = Some(GetShardMetadataRespBody { shards });
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(GetShardMetadataRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::GetShardMetadataResp(resp));
            }

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

            JournalEnginePacket::FetchOffsetReq(request) => {
                let mut resp = FetchOffsetResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::FetchOffset.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.data_handler.fetch_offset(request).await {
                    Ok(data) => {
                        resp.body = Some(data);
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: get_journal_server_code(&e),
                            error: e.to_string(),
                        });
                        resp.body = Some(FetchOffsetRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::FetchOffsetResp(resp));
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
