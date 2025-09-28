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

use crate::common::channel::RequestChannel;
use crate::common::handler::handler_process;
use crate::common::response::{response_process, ResponseProcessContext};
use crate::common::tls_acceptor::{load_certs, load_key};
use crate::context::ServerContext;
use crate::quic::acceptor::acceptor_process;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_config::broker::broker_config;
use metadata_struct::connection::NetworkConnectionType;
use protocol::codec::RobustMQCodec;
use quinn::{Endpoint, ServerConfig};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

pub struct QuicServer {
    context: ServerContext,
}

impl QuicServer {
    pub fn new(context: ServerContext) -> Self {
        QuicServer { context }
    }

    pub async fn start(&self, port: u32) -> ResultCommonError {
        let config = self.build_config()?;
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port as u16));
        let server = Endpoint::server(config, addr)?;
        let arc_quic_endpoint = Arc::new(server);
        let network_type = NetworkConnectionType::QUIC;
        let request_channel = Arc::new(RequestChannel::new(self.context.proc_config.channel_size));
        let request_recv_channel = request_channel.create_request_channel(&network_type);
        let response_recv_channel = request_channel.create_response_channel(&network_type);
        let codec = RobustMQCodec::new();
        acceptor_process(
            self.context.proc_config.accept_thread_num,
            self.context.connection_manager.clone(),
            self.context.broker_cache.clone(),
            arc_quic_endpoint.clone(),
            request_channel.clone(),
            network_type.clone(),
            codec.clone(),
            self.context.stop_sx.clone(),
        )
        .await;

        handler_process(
            self.context.proc_config.handler_process_num,
            request_recv_channel,
            self.context.connection_manager.clone(),
            self.context.command.clone(),
            request_channel.clone(),
            NetworkConnectionType::QUIC,
            self.context.stop_sx.clone(),
        )
        .await;

        response_process(ResponseProcessContext {
            response_process_num: self.context.proc_config.response_process_num,
            connection_manager: self.context.connection_manager.clone(),
            response_queue_rx: response_recv_channel,
            client_pool: self.context.client_pool.clone(),
            request_channel: request_channel.clone(),
            network_type: NetworkConnectionType::QUIC,
            stop_sx: self.context.stop_sx.clone(),
        })
        .await;

        info!("MQTT Quic Server started successfully, addr: {}", addr);
        Ok(())
    }

    pub async fn stop(&self) {}

    #[allow(clippy::result_large_err)]
    fn build_config(&self) -> Result<ServerConfig, CommonError> {
        let conf = broker_config();
        let certs = load_certs(Path::new(&conf.runtime.tls_cert))?;
        let key = load_key(Path::new(&conf.runtime.tls_key))?;
        let config = ServerConfig::with_single_cert(certs, key)?;
        Ok(config)
    }
}
