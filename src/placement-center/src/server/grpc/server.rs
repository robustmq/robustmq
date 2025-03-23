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

use axum::http;
use common_base::config::placement_center::placement_center_conf;
use common_base::tools::now_mills;
use grpc_clients::pool::ClientPool;
use log::info;
use rocksdb_engine::RocksDBEngine;

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;

use crate::core::metrics::{metrics_grpc_request_incr, metrics_grpc_request_ms};
use crate::journal::cache::JournalCacheManager;
use crate::journal::controller::call_node::JournalInnerCallManager;
use crate::mqtt::cache::MqttCacheManager;
use crate::mqtt::controller::call_broker::MQTTInnerCallManager;
use crate::route::apply::RaftMachineApply;
use crate::server::grpc::service_inner::GrpcPlacementService;
use crate::server::grpc::service_journal::GrpcEngineService;
use crate::server::grpc::service_kv::GrpcKvService;
use crate::server::grpc::service_mqtt::GrpcMqttService;
use crate::server::grpc::services_openraft::GrpcOpenRaftServices;
use protocol::placement_center::placement_center_inner::placement_center_service_server::PlacementCenterServiceServer;
use protocol::placement_center::placement_center_journal::engine_service_server::EngineServiceServer;
use protocol::placement_center::placement_center_kv::kv_service_server::KvServiceServer;
use protocol::placement_center::placement_center_mqtt::mqtt_service_server::MqttServiceServer;
use protocol::placement_center::placement_center_openraft::open_raft_service_server::OpenRaftServiceServer;
use std::pin::Pin;
use std::task::{Context, Poll};
use tonic::{transport::Server, Request, Status};
use tower::{Layer, Service};

#[allow(clippy::too_many_arguments)]
pub async fn start_grpc_server(
    raft_machine_apply: Arc<RaftMachineApply>,
    cluster_cache: Arc<PlacementCacheManager>,
    engine_cache: Arc<JournalCacheManager>,
    mqtt_cache: Arc<MqttCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    journal_call_manager: Arc<JournalInnerCallManager>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
) -> Result<(), PlacementCenterError> {
    let config = placement_center_conf();
    let ip = format!("{}:{}", config.network.local_ip, config.network.grpc_port).parse()?;

    let placement_handler = GrpcPlacementService::new(
        raft_machine_apply.clone(),
        cluster_cache.clone(),
        rocksdb_engine_handler.clone(),
        client_pool.clone(),
        journal_call_manager.clone(),
        mqtt_call_manager.clone(),
    );

    let kv_handler = GrpcKvService::new(raft_machine_apply.clone(), rocksdb_engine_handler.clone());

    let engine_handler = GrpcEngineService::new(
        raft_machine_apply.clone(),
        engine_cache.clone(),
        cluster_cache.clone(),
        rocksdb_engine_handler.clone(),
        journal_call_manager.clone(),
        client_pool.clone(),
    );

    let openraft_handler = GrpcOpenRaftServices::new(raft_machine_apply.openraft_node.clone());

    let mqtt_handler = GrpcMqttService::new(
        cluster_cache.clone(),
        mqtt_cache.clone(),
        raft_machine_apply.clone(),
        rocksdb_engine_handler.clone(),
        mqtt_call_manager.clone(),
        client_pool.clone(),
    );
    let pc_svc = PlacementCenterServiceServer::with_interceptor(placement_handler, grpc_intercept);
    let kv_svc = KvServiceServer::with_interceptor(kv_handler, grpc_intercept);
    let mqtt_svc = MqttServiceServer::with_interceptor(mqtt_handler, grpc_intercept);
    let engine_svc = EngineServiceServer::with_interceptor(engine_handler, grpc_intercept);
    let openraft_svc = OpenRaftServiceServer::with_interceptor(openraft_handler, grpc_intercept);

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
        .add_service(pc_svc)
        .add_service(kv_svc)
        .add_service(mqtt_svc)
        .add_service(engine_svc)
        .add_service(openraft_svc)
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
        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start_time = now_mills();

            // call
            let response = inner.call(req).await?;

            metrics_grpc_request_ms(now_mills() - start_time);
            Ok(response)
        })
    }
}

// See: https://github.com/hyperium/tonic/blob/master/examples/src/interceptor/server.rs
pub fn grpc_intercept(req: Request<()>) -> Result<Request<()>, Status> {
    metrics_grpc_request_incr("all");
    Ok(req)
}
