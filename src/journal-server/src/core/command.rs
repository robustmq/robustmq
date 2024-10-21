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

use grpc_clients::poll::ClientPool;
use log::error;
use protocol::journal_server::codec::JournalEnginePacket;
use protocol::journal_server::journal_engine::{
    ApiKey, ApiVersion, CreateShardResp, CreateShardRespBody, GetActiveSegmentResp,
    GetActiveSegmentRespBody, GetClusterMetadataResp, GetClusterMetadataRespBody,
    JournalEngineError, RespHeader, WriteResp, WriteRespBody,
};

use super::cache::CacheManager;
use super::handler::Handler;
use crate::server::connection::NetworkConnection;
use crate::server::connection_manager::ConnectionManager;

#[derive(Clone)]
pub struct Command {
    handler: Handler,
}

impl Command {
    pub fn new(client_poll: Arc<ClientPool>, cache_manager: Arc<CacheManager>) -> Self {
        let handler = Handler::new(cache_manager, client_poll);
        Command { handler }
    }

    pub async fn apply(
        &self,
        connect_manager: Arc<ConnectionManager>,
        tcp_connection: NetworkConnection,
        addr: SocketAddr,
        packet: JournalEnginePacket,
    ) -> Option<JournalEnginePacket> {
        match packet {
            JournalEnginePacket::GetClusterMetadataReq(request) => {
                let header = RespHeader {
                    api_key: ApiKey::GetClusterMetadata.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                let resp = GetClusterMetadataResp {
                    header: Some(header),
                    body: Some(GetClusterMetadataRespBody {
                        nodes: self.handler.get_cluster_metadata(),
                    }),
                };

                return Some(JournalEnginePacket::GetClusterMetadataResp(resp));
            }

            JournalEnginePacket::CreateShardReq(request) => {
                let mut resp = CreateShardResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::CreateShard.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.handler.create_shard(request).await {
                    Ok(replicas) => {
                        resp.body = Some(CreateShardRespBody {
                            replica_id: replicas,
                        });
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        })
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::CreateShardResp(resp));
            }

            JournalEnginePacket::GetActiveSegmentReq(request) => {
                let mut body = GetActiveSegmentRespBody::default();
                match self.handler.active_segment(request).await {
                    Ok(segments) => {
                        body.segments = segments;
                    }
                    Err(e) => {
                        body.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                    }
                }
                let resp = GetActiveSegmentResp {
                    header: None,
                    body: Some(body),
                };
                return Some(JournalEnginePacket::GetActiveSegmentResp(resp));
            }

            JournalEnginePacket::OffsetCommitReq(_) => {
                self.handler.offset_commit().await;
            }

            JournalEnginePacket::WriteReq(request) => {
                let mut body = WriteRespBody::default();
                match self.handler.write(request).await {
                    Ok(data) => {
                        body.status = data;
                    }
                    Err(e) => {
                        body.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                    }
                }
                let resp = WriteResp {
                    header: None,
                    body: Some(body),
                };
                return Some(JournalEnginePacket::WriteResp(resp));
            }

            JournalEnginePacket::ReadReq(_) => {
                self.handler.read().await;
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
