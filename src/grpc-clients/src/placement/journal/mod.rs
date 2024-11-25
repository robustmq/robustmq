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
use protocol::placement_center::placement_center_journal::engine_service_client::EngineServiceClient;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    ListShardReply, ListShardRequest, UpdateSegmentMetaReply, UpdateSegmentMetaRequest,
    UpdateSegmentStatusReply, UpdateSegmentStatusRequest,
};
use tonic::transport::Channel;

use crate::pool::ClientPool;

pub mod call;

#[derive(Debug, Clone)]
pub enum JournalServiceRequest {
    ListShard(ListShardRequest),
    CreateShard(CreateShardRequest),
    DeleteShard(DeleteShardRequest),
    ListSegment(ListSegmentRequest),
    CreateSegment(CreateNextSegmentRequest),
    DeleteSegment(DeleteSegmentRequest),
    UpdateSegmentStatus(UpdateSegmentStatusRequest),
    ListSegmentMeta(ListSegmentMetaRequest),
    UpdateSegmentMeta(UpdateSegmentMetaRequest),
}

#[derive(Debug, Clone)]
pub enum JournalServiceReply {
    ListShard(ListShardReply),
    CreateShard(CreateShardReply),
    DeleteShard(DeleteShardReply),
    ListSegment(ListSegmentReply),
    CreateSegment(CreateNextSegmentReply),
    DeleteSegment(DeleteSegmentReply),
    UpdateSegmentStatus(UpdateSegmentStatusReply),
    ListSegmentMeta(ListSegmentMetaReply),
    UpdateSegmentMeta(UpdateSegmentMetaReply),
}

pub(super) async fn call_journal_service_once(
    client_pool: &ClientPool,
    addr: &str,
    request: JournalServiceRequest,
) -> Result<JournalServiceReply, CommonError> {
    use JournalServiceRequest::*;

    match request {
        ListShard(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.list_shard(request).await?;
            Ok(JournalServiceReply::ListShard(reply.into_inner()))
        }
        CreateShard(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.create_shard(request).await?;
            Ok(JournalServiceReply::CreateShard(reply.into_inner()))
        }
        DeleteShard(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.delete_shard(request).await?;
            Ok(JournalServiceReply::DeleteShard(reply.into_inner()))
        }
        ListSegment(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.list_segment(request).await?;
            Ok(JournalServiceReply::ListSegment(reply.into_inner()))
        }
        CreateSegment(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.create_next_segment(request).await?;
            Ok(JournalServiceReply::CreateSegment(reply.into_inner()))
        }
        DeleteSegment(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.delete_segment(request).await?;
            Ok(JournalServiceReply::DeleteSegment(reply.into_inner()))
        }
        UpdateSegmentStatus(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.update_segment_status(request).await?;
            Ok(JournalServiceReply::UpdateSegmentStatus(reply.into_inner()))
        }
        ListSegmentMeta(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.list_segment_meta(request).await?;
            Ok(JournalServiceReply::ListSegmentMeta(reply.into_inner()))
        }
        UpdateSegmentMeta(request) => {
            let mut client = client_pool.placement_center_journal_services_client(addr).await?;
            let reply = client.update_segment_meta(request).await?;
            Ok(JournalServiceReply::UpdateSegmentMeta(reply.into_inner()))
        }
    }
}

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
