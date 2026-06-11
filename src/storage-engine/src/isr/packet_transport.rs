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

use crate::clients::manager::ClientConnectionManager;
use crate::clients::packet::build_fetch_req;
use crate::core::error::StorageEngineError;
use crate::isr::fetcher::FetchTransport;
use async_trait::async_trait;
use protocol::storage::codec::StorageEnginePacket;
use protocol::storage::protocol::{
    FetchReqBody, FetchRespBody, OffsetsForLeaderEpochReq, OffsetsForLeaderEpochReqBody,
    OffsetsForLeaderEpochRespBody,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PacketFetchTransport {
    client: Arc<ClientConnectionManager>,
}

impl PacketFetchTransport {
    pub fn new(client: Arc<ClientConnectionManager>) -> Self {
        PacketFetchTransport { client }
    }
}

#[async_trait]
impl FetchTransport for PacketFetchTransport {
    async fn fetch(
        &self,
        leader_node_id: u64,
        req: FetchReqBody,
    ) -> Result<FetchRespBody, StorageEngineError> {
        let packet = StorageEnginePacket::FetchReq(build_fetch_req(req));
        match self.client.read_send(leader_node_id, packet).await? {
            StorageEnginePacket::FetchResp(resp) => Ok(resp.body),
            other => Err(StorageEngineError::CommonErrorStr(format!(
                "fetch to node {leader_node_id} expected FetchResp, got {other}"
            ))),
        }
    }

    async fn offsets_for_leader_epoch(
        &self,
        leader_node_id: u64,
        req: OffsetsForLeaderEpochReqBody,
    ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
        let packet =
            StorageEnginePacket::OffsetsForLeaderEpochReq(OffsetsForLeaderEpochReq::new(req));
        match self.client.read_send(leader_node_id, packet).await? {
            StorageEnginePacket::OffsetsForLeaderEpochResp(resp) => Ok(resp.body),
            other => Err(StorageEngineError::CommonErrorStr(format!(
                "offsets_for_leader_epoch to node {leader_node_id} expected resp, got {other}"
            ))),
        }
    }
}
