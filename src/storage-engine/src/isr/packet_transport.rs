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
use crate::core::error::StorageEngineError;
use crate::isr::fetcher::FetchTransport;
use async_trait::async_trait;
use protocol::storage::protocol::{
    FetchReqBody, FetchRespBody, OffsetsForLeaderEpochReqBody, OffsetsForLeaderEpochRespBody,
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
        self.client.send_fetch(leader_node_id, req).await
    }

    async fn offsets_for_leader_epoch(
        &self,
        leader_node_id: u64,
        req: OffsetsForLeaderEpochReqBody,
    ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
        self.client
            .send_offsets_for_leader_epoch(leader_node_id, req)
            .await
    }
}
