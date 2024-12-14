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
use protocol::placement_center::placement_center_kv::kv_service_client::KvServiceClient;
use protocol::placement_center::placement_center_kv::{
    DeleteReply, DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetReply,
    SetRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct KvServiceManager {
    pub addr: String,
}

impl KvServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for KvServiceManager {
    type Connection = KvServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match KvServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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
    SetRequest,
    KvServiceClient<Channel>,
    SetReply,
    placement_center_kv_services_client,
    set,
    true
);

impl_retriable_request!(
    GetRequest,
    KvServiceClient<Channel>,
    GetReply,
    placement_center_kv_services_client,
    get,
    true
);

impl_retriable_request!(
    DeleteRequest,
    KvServiceClient<Channel>,
    DeleteReply,
    placement_center_kv_services_client,
    delete,
    true
);

impl_retriable_request!(
    ExistsRequest,
    KvServiceClient<Channel>,
    ExistsReply,
    placement_center_kv_services_client,
    exists,
    true
);

#[cfg(test)]
mod tests {}
