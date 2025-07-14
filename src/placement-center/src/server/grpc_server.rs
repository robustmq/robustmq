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

use crate::controller::journal::call_node::JournalInnerCallManager;
use crate::controller::mqtt::call_broker::MQTTInnerCallManager;
use crate::core::cache::CacheManager;
use crate::core::error::PlacementCenterError;
use crate::core::metrics::{metrics_grpc_request_incr, metrics_grpc_request_ms};
use crate::raft::route::apply::RaftMachineApply;
use crate::server::service_inner::GrpcPlacementService;
use crate::server::service_journal::GrpcEngineService;
use crate::server::service_kv::GrpcKvService;
use crate::server::service_mqtt::GrpcMqttService;
use crate::server::service_openraft::GrpcOpenRaftServices;
use axum::http::{self};
use common_base::tools::now_mills;
use common_config::place::config::placement_center_conf;
use grpc_clients::pool::ClientPool;
use protocol::placement_center::placement_center_inner::placement_center_service_server::PlacementCenterServiceServer;
use protocol::placement_center::placement_center_journal::engine_service_server::EngineServiceServer;
use protocol::placement_center::placement_center_kv::kv_service_server::KvServiceServer;
use protocol::placement_center::placement_center_mqtt::mqtt_service_server::MqttServiceServer;
use protocol::placement_center::placement_center_openraft::open_raft_service_server::OpenRaftServiceServer;
use rocksdb_engine::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::transport::Server;
use tower::{Layer, Service};
use tracing::info;

#[allow(clippy::too_many_arguments)]
pub async fn start_grpc_server(
    raft_machine_apply: Arc<RaftMachineApply>,
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
) -> Result<(), PlacementCenterError> {
    let config = placement_center_conf();
    let ip = format!("{}:{}", config.network.local_ip, config.network.grpc_port).parse()?;

    let placement_handler = GrpcPlacementService::new(
        raft_machine_apply.clone(),
        cache_manager.clone(),
        rocksdb_engine_handler.clone(),
        client_pool.clone(),
        journal_call_manager.clone(),
        mqtt_call_manager.clone(),
    );

    let kv_handler = GrpcKvService::new(raft_machine_apply.clone(), rocksdb_engine_handler.clone());

    let engine_handler = GrpcEngineService::new(
        raft_machine_apply.clone(),
        cache_manager.clone(),
        rocksdb_engine_handler.clone(),
        journal_call_manager.clone(),
        client_pool.clone(),
    );

    let openraft_handler = GrpcOpenRaftServices::new(raft_machine_apply.openraft_node.clone());

    let mqtt_handler = GrpcMqttService::new(
        cache_manager.clone(),
        raft_machine_apply.clone(),
        rocksdb_engine_handler.clone(),
        mqtt_call_manager.clone(),
        client_pool.clone(),
    );

    let grpc_max_decoding_message_size = config.network.grpc_max_decoding_message_size as usize;

    let layer = tower::ServiceBuilder::new()
        .layer(BaseMiddlewareLayer::default())
        .into_inner();

    // allow cors for development or production(maybe)?
    let cors_layer = tower_http::cors::CorsLayer::very_permissive();

    info!("RobustMQ Meta Grpc Server start success. bind addr:{}", ip);
    Server::builder()
        .accept_http1(true)
        .layer(cors_layer)
        .layer(tonic_web::GrpcWebLayer::new())
        .layer(layer)
        .add_service(
            PlacementCenterServiceServer::new(placement_handler)
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            KvServiceServer::new(kv_handler)
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            MqttServiceServer::new(mqtt_handler)
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            EngineServiceServer::new(engine_handler)
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            OpenRaftServiceServer::new(openraft_handler)
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .serve(ip)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct BaseMiddlewareLayer {}

impl<S> Layer<S> for BaseMiddlewareLayer {
    type Service = BaseMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        BaseMiddleware { inner: service }
    }
}

// See: https://github.com/hyperium/tonic/blob/master/examples/src/tower/server.rs
#[derive(Debug, Clone)]
struct BaseMiddleware<S> {
    inner: S,
}

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for BaseMiddleware<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let paths = parse_path(req.uri().path());
        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start_time = now_mills();

            // call
            let response = inner.call(req).await?;

            metrics_grpc_request_ms(
                paths.0.as_str(),
                paths.1.as_str(),
                (now_mills() - start_time) as f64,
            );
            metrics_grpc_request_incr(paths.0.as_str(), paths.1.as_str());
            Ok(response)
        })
    }
}

fn parse_path(uri: &str) -> (String, String) {
    let paths: Vec<&str> = uri.split("/").collect();
    (paths[1].to_string(), paths[2].to_string())
}

#[cfg(test)]
mod test {

    use crate::server::grpc_server::parse_path;

    #[tokio::test]
    async fn parse_path_test() {
        let path = "/placement.center.kv.KvService/exists";
        let paths = parse_path(path);
        assert_eq!(paths.0, "placement.center.kv.KvService");
        assert_eq!(paths.1, "exists");

        let path = "/placement.center.kv.KvService/get";
        let paths = parse_path(path);
        assert_eq!(paths.0, "placement.center.kv.KvService");
        assert_eq!(paths.1, "get");
    }
}
