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

use std::sync::Arc;

use common_base::error::common::CommonError;
use grpc_clients::poll::ClientPool;
use log::info;
use protocol::journal_server::journal_admin::journal_server_admin_service_server::JournalServerAdminServiceServer;
use protocol::journal_server::journal_inner::journal_server_inner_service_server::JournalServerInnerServiceServer;
use tonic::transport::Server;

use crate::server::grpc::admin::GrpcJournalServerAdminService;
use crate::server::grpc::inner::GrpcJournalServerInnerService;

pub struct GrpcServer {
    port: u32,
    _client_poll: Arc<ClientPool>,
}

impl GrpcServer {
    pub fn new(port: u32, client_poll: Arc<ClientPool>) -> Self {
        Self {
            port,
            _client_poll: client_poll,
        }
    }
    pub async fn start(&self) -> Result<(), CommonError> {
        let addr = format!("0.0.0.0:{}", self.port).parse()?;
        info!(
            "Journal Engine Grpc Server start success. port:{}",
            self.port
        );
        let admin_handler = GrpcJournalServerAdminService::new();
        let inner_handler = GrpcJournalServerInnerService::new();

        Server::builder()
            .add_service(JournalServerAdminServiceServer::new(admin_handler))
            .add_service(JournalServerInnerServiceServer::new(inner_handler))
            .serve(addr)
            .await?;
        Ok(())
    }
}
