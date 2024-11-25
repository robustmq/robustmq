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

use crate::pool::ClientPool;

pub mod call;

#[derive(Debug, Clone)]
pub enum KvServiceRequest {
    Set(SetRequest),
    Get(GetRequest),
    Delete(DeleteRequest),
    Exists(ExistsRequest),
}

#[derive(Debug, Clone)]
pub enum KvServiceReply {
    Set(SetReply),
    Get(GetReply),
    Delete(DeleteReply),
    Exists(ExistsReply),
}

pub(super) async fn call_kv_service_once(
    client_pool: &ClientPool,
    addr: &str,
    request: KvServiceRequest,
) -> Result<KvServiceReply, CommonError> {
    use KvServiceRequest::*;

    match request {
        Set(request) => {
            let mut client = client_pool.placement_center_kv_services_client(addr).await?;
            let reply = client.set(request).await?;
            Ok(KvServiceReply::Set(reply.into_inner()))
        }
        Get(request) => {
            let mut client = client_pool.placement_center_kv_services_client(addr).await?;
            let reply = client.get(request).await?;
            Ok(KvServiceReply::Get(reply.into_inner()))
        }
        Delete(request) => {
            let mut client = client_pool.placement_center_kv_services_client(addr).await?;
            let reply = client.delete(request).await?;
            Ok(KvServiceReply::Delete(reply.into_inner()))
        }
        Exists(request) => {
            let mut client = client_pool.placement_center_kv_services_client(addr).await?;
            let reply = client.exists(request).await?;
            Ok(KvServiceReply::Exists(reply.into_inner()))
        }
    }
}

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

#[cfg(test)]
mod tests {}
