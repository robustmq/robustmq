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

use common_base::error::common::CommonError;
use mobc::Manager;
use protocol::meta::placement_center_journal::engine_service_client::EngineServiceClient;
use protocol::meta::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    ListShardReply, ListShardRequest, UpdateSegmentMetaReply, UpdateSegmentMetaRequest,
    UpdateSegmentStatusReply, UpdateSegmentStatusRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct JournalServiceManager {
    pub addr: String,
}

impl JournalServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for JournalServiceManager {
    type Connection = EngineServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match EngineServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommonError(format!(
                    "{},{}",
                    err,
                    self.addr.clone()
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

impl_retriable_request!(
    ListShardRequest,
    EngineServiceClient<Channel>,
    ListShardReply,
    placement_center_journal_services_client,
    list_shard,
    true
);

impl_retriable_request!(
    CreateShardRequest,
    EngineServiceClient<Channel>,
    CreateShardReply,
    placement_center_journal_services_client,
    create_shard,
    true
);

impl_retriable_request!(
    DeleteShardRequest,
    EngineServiceClient<Channel>,
    DeleteShardReply,
    placement_center_journal_services_client,
    delete_shard,
    true
);

impl_retriable_request!(
    ListSegmentRequest,
    EngineServiceClient<Channel>,
    ListSegmentReply,
    placement_center_journal_services_client,
    list_segment,
    true
);

impl_retriable_request!(
    CreateNextSegmentRequest,
    EngineServiceClient<Channel>,
    CreateNextSegmentReply,
    placement_center_journal_services_client,
    create_next_segment,
    true
);

impl_retriable_request!(
    DeleteSegmentRequest,
    EngineServiceClient<Channel>,
    DeleteSegmentReply,
    placement_center_journal_services_client,
    delete_segment,
    true
);

impl_retriable_request!(
    UpdateSegmentStatusRequest,
    EngineServiceClient<Channel>,
    UpdateSegmentStatusReply,
    placement_center_journal_services_client,
    update_segment_status,
    true
);

impl_retriable_request!(
    ListSegmentMetaRequest,
    EngineServiceClient<Channel>,
    ListSegmentMetaReply,
    placement_center_journal_services_client,
    list_segment_meta,
    true
);

impl_retriable_request!(
    UpdateSegmentMetaRequest,
    EngineServiceClient<Channel>,
    UpdateSegmentMetaReply,
    placement_center_journal_services_client,
    update_segment_meta,
    true
);
