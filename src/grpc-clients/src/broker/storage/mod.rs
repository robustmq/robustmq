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

use crate::macros::impl_retriable_request;
use common_base::error::common::CommonError;
use mobc::Manager;
use protocol::broker::broker_storage::{
    broker_storage_service_client::BrokerStorageServiceClient, DeleteSegmentFileReply,
    DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest,
};
use tonic::transport::Channel;

pub mod call;

#[derive(Clone)]
pub struct BrokerStorageServiceManager {
    pub addr: String,
}

impl BrokerStorageServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for BrokerStorageServiceManager {
    type Connection = BrokerStorageServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match BrokerStorageServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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
    DeleteShardFileRequest,
    BrokerStorageServiceClient<Channel>,
    DeleteShardFileReply,
    broker_storage_services_client,
    delete_shard_file,
    "JournalInnerService",
    "DeleteShardFile"
);

impl_retriable_request!(
    GetShardDeleteStatusRequest,
    BrokerStorageServiceClient<Channel>,
    GetShardDeleteStatusReply,
    broker_storage_services_client,
    get_shard_delete_status,
    "JournalInnerService",
    "GetShardDeleteStatus"
);

impl_retriable_request!(
    DeleteSegmentFileRequest,
    BrokerStorageServiceClient<Channel>,
    DeleteSegmentFileReply,
    broker_storage_services_client,
    delete_segment_file,
    "JournalInnerService",
    "DeleteSegmentFile"
);

impl_retriable_request!(
    GetSegmentDeleteStatusRequest,
    BrokerStorageServiceClient<Channel>,
    GetSegmentDeleteStatusReply,
    broker_storage_services_client,
    get_segment_delete_status,
    "JournalInnerService",
    "GetSegmentDeleteStatus"
);
