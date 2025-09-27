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

use crate::cluster_service::ClusterInnerService;
use axum::http::{self};
use common_base::error::common::CommonError;
use common_base::tools::now_mills;
use common_config::broker::broker_config;
use common_metrics::grpc::{extract_grpc_status_code, parse_grpc_path, record_grpc_request};
use journal_server::server::grpc::admin::GrpcJournalServerAdminService;
use journal_server::server::grpc::inner::GrpcJournalServerInnerService;
use journal_server::JournalServerParams;
use meta_service::server::service_inner::GrpcPlacementService;
use meta_service::server::service_journal::GrpcEngineService;
use meta_service::server::service_kv::GrpcKvService;
use meta_service::server::service_mqtt::GrpcMqttService;
use meta_service::server::service_raft::GrpcOpenRaftServices;
use meta_service::MetaServiceServerParams;
use mqtt_broker::broker::MqttBrokerServerParams;
use mqtt_broker::server::inner::GrpcInnerServices;
use protocol::broker::broker_mqtt_inner::mqtt_broker_inner_service_server::MqttBrokerInnerServiceServer;
use protocol::cluster::cluster_status::cluster_service_server::ClusterServiceServer;
use protocol::journal::journal_admin::journal_server_admin_service_server::JournalServerAdminServiceServer;
use protocol::journal::journal_inner::journal_server_inner_service_server::JournalServerInnerServiceServer;
use protocol::meta::meta_service_inner::meta_service_service_server::MetaServiceServiceServer;
use protocol::meta::meta_service_journal::engine_service_server::EngineServiceServer;
use protocol::meta::meta_service_kv::kv_service_server::KvServiceServer;
use protocol::meta::meta_service_mqtt::mqtt_service_server::MqttServiceServer;
use protocol::meta::meta_service_openraft::open_raft_service_server::OpenRaftServiceServer;
use std::pin::Pin;
use std::task::{Context, Poll};
use tonic::transport::Server;
use tower::{Layer, Service};
use tracing::info;

pub async fn start_grpc_server(
    place_params: MetaServiceServerParams,
    mqtt_params: MqttBrokerServerParams,
    journal_params: JournalServerParams,
    grpc_port: u32,
) -> Result<(), CommonError> {
    let ip = format!("0.0.0.0:{grpc_port}").parse()?;
    let cors_layer = tower_http::cors::CorsLayer::very_permissive();
    let layer = tower::ServiceBuilder::new()
        .layer(BaseMiddlewareLayer::default())
        .into_inner();

    let grpc_max_decoding_message_size = 268435456;
    info!("Broker Grpc Server start success. addr:{}", ip);
    let mut route = Server::builder()
        .accept_http1(true)
        .layer(cors_layer)
        .layer(tonic_web::GrpcWebLayer::new())
        .layer(layer)
        .add_service(
            ClusterServiceServer::new(ClusterInnerService::new())
                .max_decoding_message_size(grpc_max_decoding_message_size),
        );

    let config = broker_config();
    if config.is_start_meta() {
        route = route
            .add_service(
                MetaServiceServiceServer::new(get_place_inner_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            )
            .add_service(
                KvServiceServer::new(get_place_kv_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            )
            .add_service(
                MqttServiceServer::new(get_place_mqtt_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            )
            .add_service(
                EngineServiceServer::new(get_place_engine_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            )
            .add_service(
                OpenRaftServiceServer::new(get_place_raft_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            );
    }

    if config.is_start_broker() {
        route = route.add_service(
            MqttBrokerInnerServiceServer::new(get_mqtt_inner_handler(&mqtt_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        );
    }

    if config.is_start_journal() {
        route = route
            .add_service(
                JournalServerAdminServiceServer::new(get_journal_admin_handler(&journal_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            )
            .add_service(
                JournalServerInnerServiceServer::new(get_journal_inner_handler(&journal_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            );
    }

    route.serve(ip).await?;
    Ok(())
}

fn get_place_inner_handler(place_params: &MetaServiceServerParams) -> GrpcPlacementService {
    GrpcPlacementService::new(
        place_params.storage_driver.clone(),
        place_params.cache_manager.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.client_pool.clone(),
        place_params.journal_call_manager.clone(),
        place_params.mqtt_call_manager.clone(),
    )
}

fn get_place_kv_handler(place_params: &MetaServiceServerParams) -> GrpcKvService {
    GrpcKvService::new(
        place_params.storage_driver.clone(),
        place_params.rocksdb_engine_handler.clone(),
    )
}

fn get_place_mqtt_handler(place_params: &MetaServiceServerParams) -> GrpcMqttService {
    GrpcMqttService::new(
        place_params.cache_manager.clone(),
        place_params.storage_driver.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.mqtt_call_manager.clone(),
        place_params.client_pool.clone(),
    )
}

fn get_place_engine_handler(place_params: &MetaServiceServerParams) -> GrpcEngineService {
    GrpcEngineService::new(
        place_params.storage_driver.clone(),
        place_params.cache_manager.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.journal_call_manager.clone(),
        place_params.client_pool.clone(),
    )
}

fn get_place_raft_handler(place_params: &MetaServiceServerParams) -> GrpcOpenRaftServices {
    GrpcOpenRaftServices::new(place_params.storage_driver.raft_node.clone())
}

fn get_mqtt_inner_handler(mqtt_params: &MqttBrokerServerParams) -> GrpcInnerServices {
    GrpcInnerServices::new(
        mqtt_params.cache_manager.clone(),
        mqtt_params.subscribe_manager.clone(),
        mqtt_params.connector_manager.clone(),
        mqtt_params.schema_manager.clone(),
        mqtt_params.client_pool.clone(),
        mqtt_params.message_storage_adapter.clone(),
    )
}

fn get_journal_admin_handler(params: &JournalServerParams) -> GrpcJournalServerAdminService {
    GrpcJournalServerAdminService::new(params.cache_manager.clone())
}

fn get_journal_inner_handler(params: &JournalServerParams) -> GrpcJournalServerInnerService {
    GrpcJournalServerInnerService::new(
        params.cache_manager.clone(),
        params.segment_file_manager.clone(),
        params.rocksdb_engine_handler.clone(),
    )
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
        let (service, method) = parse_grpc_path(req.uri().path())
            .unwrap_or_else(|_| ("unknown".to_string(), "unknown".to_string()));

        let request_size = req
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok());

        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start_time = now_mills();
            let response = inner.call(req).await;
            let duration_ms = (now_mills() - start_time) as f64;

            match response {
                Ok(resp) => {
                    let response_size = resp
                        .headers()
                        .get("content-length")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<f64>().ok());
                    let status_code = extract_grpc_status_code(resp.headers());
                    record_grpc_request(
                        service,
                        method,
                        status_code,
                        duration_ms,
                        request_size,
                        response_size,
                    );

                    Ok(resp)
                }
                Err(err) => {
                    record_grpc_request(
                        service,
                        method,
                        "INTERNAL".to_string(),
                        duration_ms,
                        request_size,
                        None,
                    );

                    Err(err)
                }
            }
        })
    }
}

#[cfg(test)]
mod test {}
