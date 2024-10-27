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

use crate::core::cache::CacheManager;
use crate::server::grpc::admin::GrpcJournalServerAdminService;
use crate::server::grpc::inner::GrpcJournalServerInnerService;

pub struct GrpcServer {
    port: u32,
    client_poll: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    tls_config: Option<tonic::transport::ServerTlsConfig>,
}

impl GrpcServer {
    pub fn new(port: u32, client_poll: Arc<ClientPool>, cache_manager: Arc<CacheManager>) -> Self {
        Self {
            port,
            client_poll,
            cache_manager,
            tls_config: None,
        }
    }

    pub fn new_with_tls(
        port: u32,
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
        tls_config: tonic::transport::ServerTlsConfig,
    ) -> Self {
        Self {
            port,
            client_poll: client_pool,
            cache_manager,
            tls_config: Some(tls_config),
        }
    }

    pub async fn start(&self) -> Result<(), CommonError> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), self.port);
        info!(
            "Journal Engine Grpc Server start success. port:{}",
            self.port
        );
        let admin_handler = GrpcJournalServerAdminService::new();
        let inner_handler = GrpcJournalServerInnerService::new(self.cache_manager.clone());

        let mut server_builder = Server::builder();

        if let Some(tls_config) = &self.tls_config {
            server_builder = server_builder.tls_config(tls_config.clone())?;
        }

        server_builder
            .add_service(JournalServerAdminServiceServer::new(admin_handler))
            .add_service(JournalServerInnerServiceServer::new(inner_handler))
            .serve(addr)
            .await?;
        Ok(())
    }
}
