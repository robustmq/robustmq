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

use crate::cluster_service::GrpcBrokerCommonService;
use axum::http::{self};
use common_base::error::common::CommonError;
use common_base::role::{is_broker_node, is_engine_node, is_meta_node};
use common_base::tools::now_millis;
use common_config::broker::broker_config;
use common_metrics::grpc::{extract_grpc_status_code, parse_grpc_path, record_grpc_request};
use meta_service::server::service_common::GrpcPlacementService;
use meta_service::server::service_engine::GrpcEngineService;
use meta_service::server::service_mqtt::GrpcMqttService;
use meta_service::MetaServiceServerParams;
use mqtt_broker::broker::MqttBrokerServerParams;
use mqtt_broker::server::inner::GrpcInnerServices;
use protocol::broker::broker_common::broker_common_service_server::BrokerCommonServiceServer;
use protocol::broker::broker_mqtt::broker_mqtt_service_server::BrokerMqttServiceServer;
use protocol::broker::broker_storage::broker_storage_service_server::BrokerStorageServiceServer;
use protocol::meta::meta_service_common::meta_service_service_server::MetaServiceServiceServer;
use protocol::meta::meta_service_journal::engine_service_server::EngineServiceServer;
use protocol::meta::meta_service_mqtt::mqtt_service_server::MqttServiceServer;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use storage_engine::server::inner::GrpcBrokerStorageServerService;
use storage_engine::StorageEngineParams;
use tonic::transport::Server;
use tower::{Layer, Service};
use tracing::{info, warn};

const SLOW_GRPC_WARN_THRESHOLD_MS: f64 = 2000.0;

pub async fn start_grpc_server(
    place_params: MetaServiceServerParams,
    mqtt_params: MqttBrokerServerParams,
    engine_params: StorageEngineParams,
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
        .http2_keepalive_interval(Some(Duration::from_secs(30)))
        .http2_keepalive_timeout(Some(Duration::from_secs(60)))
        .timeout(Duration::from_secs(60))
        .layer(cors_layer)
        .layer(tonic_web::GrpcWebLayer::new())
        .layer(layer)
        .add_service(
            BrokerCommonServiceServer::new(GrpcBrokerCommonService::new(
                mqtt_params.clone(),
                engine_params.clone(),
            ))
            .max_decoding_message_size(grpc_max_decoding_message_size),
        );

    let config = broker_config();
    if is_meta_node(&config.roles) {
        route = route
            .add_service(
                MetaServiceServiceServer::new(get_place_inner_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            )
            .add_service(
                MqttServiceServer::new(get_place_mqtt_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            )
            .add_service(
                EngineServiceServer::new(get_place_engine_handler(&place_params))
                    .max_decoding_message_size(grpc_max_decoding_message_size),
            );
    }

    if is_broker_node(&config.roles) {
        route = route.add_service(
            BrokerMqttServiceServer::new(get_mqtt_inner_handler(&mqtt_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        );
    }

    if is_engine_node(&config.roles) {
        route = route.add_service(
            BrokerStorageServiceServer::new(get_storage_engine_inner_handler(&engine_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        );
    }

    route.serve(ip).await?;
    Ok(())
}

fn get_place_inner_handler(place_params: &MetaServiceServerParams) -> GrpcPlacementService {
    GrpcPlacementService::new(
        place_params.raft_manager.clone(),
        place_params.cache_manager.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.client_pool.clone(),
        place_params.call_manager.clone(),
    )
}

fn get_place_mqtt_handler(place_params: &MetaServiceServerParams) -> GrpcMqttService {
    GrpcMqttService::new(
        place_params.cache_manager.clone(),
        place_params.raft_manager.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.call_manager.clone(),
        place_params.client_pool.clone(),
    )
}

fn get_place_engine_handler(place_params: &MetaServiceServerParams) -> GrpcEngineService {
    GrpcEngineService::new(
        place_params.raft_manager.clone(),
        place_params.cache_manager.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.call_manager.clone(),
        place_params.client_pool.clone(),
    )
}

fn get_mqtt_inner_handler(mqtt_params: &MqttBrokerServerParams) -> GrpcInnerServices {
    GrpcInnerServices::new(
        mqtt_params.cache_manager.clone(),
        mqtt_params.subscribe_manager.clone(),
        mqtt_params.client_pool.clone(),
        mqtt_params.storage_driver_manager.clone(),
        mqtt_params.retain_message_manager.clone(),
    )
}

fn get_storage_engine_inner_handler(
    _params: &StorageEngineParams,
) -> GrpcBrokerStorageServerService {
    GrpcBrokerStorageServerService::new()
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

        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start_time = now_millis();
            let response = inner.call(req).await;
            let duration_ms = (now_millis() - start_time) as f64;

            match response {
                Ok(resp) => {
                    let status_code = extract_grpc_status_code(resp.headers());

                    if duration_ms > SLOW_GRPC_WARN_THRESHOLD_MS {
                        warn!(
                            "Slow gRPC request. service={}, method={}, status={}, duration_ms={:.2}",
                            service, method, status_code, duration_ms
                        );
                    }

                    record_grpc_request(&service, &method, &status_code, duration_ms);
                    Ok(resp)
                }
                Err(err) => {
                    warn!(
                        "gRPC request failed. service={}, method={}, duration_ms={:.2}",
                        service, method, duration_ms
                    );

                    record_grpc_request(&service, &method, "INTERNAL", duration_ms);
                    Err(err)
                }
            }
        })
    }
}

#[cfg(test)]
mod test {}
