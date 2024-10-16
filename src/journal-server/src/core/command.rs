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

use log::error;
use protocol::journal_server::codec::JournalEnginePacket;
use protocol::journal_server::journal_engine::{
    GetActiveSegmentResp, GetActiveSegmentRespBody, GetClusterMetadataResp,
    GetClusterMetadataRespBody, JournalEngineError,
};

use super::cache::CacheManager;
use super::handler::Handler;
use crate::server::connection::NetworkConnection;
use crate::server::connection_manager::ConnectionManager;

#[derive(Debug, Clone)]
pub struct Command {
    handler: Handler,
}

impl Command {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        let handler = Handler::new(cache_manager);
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
                let resp = GetClusterMetadataResp {
                    header: None,
                    body: Some(GetClusterMetadataRespBody {
                        nodes: self.handler.get_cluster_metadata(),
                    }),
                };

                return Some(JournalEnginePacket::GetClusterMetadataResp(resp));
            }

            JournalEnginePacket::GetActiveSegmentReq(request) => {
                let mut resp = GetActiveSegmentResp::default();
                let mut body = GetActiveSegmentRespBody::default();
                match self.handler.active_segment(request).await {
                    Ok(addrs) => {
                        body.replica_id = addrs.replica.iter().map(|node| node.node_id).collect();
                    }
                    Err(e) => {
                        body.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                    }
                }
                resp.body = Some(body);
                return Some(JournalEnginePacket::GetActiveSegmentResp(resp));
            }

            JournalEnginePacket::OffsetCommitReq(_) => {
                self.handler.offset_commit().await;
            }

            JournalEnginePacket::WriteReq(_) => {
                self.handler.write().await;
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
