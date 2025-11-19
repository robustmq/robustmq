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
use protocol::journal::journal_admin::journal_server_admin_service_client::JournalServerAdminServiceClient;
use protocol::journal::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct JournalAdminServiceManager {
    pub addr: String,
}

impl JournalAdminServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for JournalAdminServiceManager {
    type Connection = JournalServerAdminServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match JournalServerAdminServiceClient::connect(format!("http://{}", self.addr.clone()))
            .await
        {
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
    JournalServerAdminServiceClient<Channel>,
    ListShardReply,
    journal_admin_services_client,
    list_shard,
    "JournalAdminService",
    "ListShard"
);

impl_retriable_request!(
    ListSegmentRequest,
    JournalServerAdminServiceClient<Channel>,
    ListSegmentReply,
    journal_admin_services_client,
    list_segment,
    "JournalAdminService",
    "ListSegment"
);
