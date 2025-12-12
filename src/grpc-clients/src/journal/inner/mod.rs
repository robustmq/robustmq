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
use protocol::journal::journal_inner::journal_server_inner_service_client::JournalServerInnerServiceClient;
use protocol::journal::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct JournalInnerServiceManager {
    pub addr: String,
}

impl JournalInnerServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for JournalInnerServiceManager {
    type Connection = JournalServerInnerServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match JournalServerInnerServiceClient::connect(format!("http://{}", self.addr)).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => return Err(CommonError::CommonError(format!("{},{}", err, self.addr))),
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

impl_retriable_request!(
    UpdateJournalCacheRequest,
    JournalServerInnerServiceClient<Channel>,
    UpdateJournalCacheReply,
    journal_inner_services_client,
    update_cache,
    "JournalInnerService",
    "UpdateCache"
);

impl_retriable_request!(
    DeleteShardFileRequest,
    JournalServerInnerServiceClient<Channel>,
    DeleteShardFileReply,
    journal_inner_services_client,
    delete_shard_file,
    "JournalInnerService",
    "DeleteShardFile"
);

impl_retriable_request!(
    GetShardDeleteStatusRequest,
    JournalServerInnerServiceClient<Channel>,
    GetShardDeleteStatusReply,
    journal_inner_services_client,
    get_shard_delete_status,
    "JournalInnerService",
    "GetShardDeleteStatus"
);

impl_retriable_request!(
    DeleteSegmentFileRequest,
    JournalServerInnerServiceClient<Channel>,
    DeleteSegmentFileReply,
    journal_inner_services_client,
    delete_segment_file,
    "JournalInnerService",
    "DeleteSegmentFile"
);

impl_retriable_request!(
    GetSegmentDeleteStatusRequest,
    JournalServerInnerServiceClient<Channel>,
    GetSegmentDeleteStatusReply,
    journal_inner_services_client,
    get_segment_delete_status,
    "JournalInnerService",
    "GetSegmentDeleteStatus"
);
