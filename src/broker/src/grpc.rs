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

use crate::metrics::{metrics_grpc_request_incr, metrics_grpc_request_ms};
use axum::http::{self};
use common_base::error::common::CommonError;
use common_base::tools::now_mills;
use journal_server::server::grpc::admin::GrpcJournalServerAdminService;
use journal_server::server::grpc::inner::GrpcJournalServerInnerService;
use journal_server::JournalServerParams;
use mqtt_broker::broker::MqttBrokerServerParams;
use mqtt_broker::server::grpc::admin::GrpcAdminServices;
use mqtt_broker::server::grpc::inner::GrpcInnerServices;
use placement_center::server::service_inner::GrpcPlacementService;
use placement_center::server::service_journal::GrpcEngineService;
use placement_center::server::service_kv::GrpcKvService;
use placement_center::server::service_mqtt::GrpcMqttService;
use placement_center::server::service_raft::GrpcOpenRaftServices;
use placement_center::PlacementCenterServerParams;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminServiceServer;
use protocol::broker_mqtt::broker_mqtt_inner::mqtt_broker_inner_service_server::MqttBrokerInnerServiceServer;
use protocol::journal_server::journal_admin::journal_server_admin_service_server::JournalServerAdminServiceServer;
use protocol::journal_server::journal_inner::journal_server_inner_service_server::JournalServerInnerServiceServer;
// use protocol::journal_server::journal_admin::journal_server_admin_service_server::JournalServerAdminServiceServer;
// use protocol::journal_server::journal_inner::journal_server_inner_service_server::JournalServerInnerServiceServer;
use protocol::placement_center::placement_center_inner::placement_center_service_server::PlacementCenterServiceServer;
use protocol::placement_center::placement_center_journal::engine_service_server::EngineServiceServer;
use protocol::placement_center::placement_center_kv::kv_service_server::KvServiceServer;
use protocol::placement_center::placement_center_mqtt::mqtt_service_server::MqttServiceServer;
use protocol::placement_center::placement_center_openraft::open_raft_service_server::OpenRaftServiceServer;
use std::pin::Pin;
use std::task::{Context, Poll};
use tonic::transport::Server;
use tower::{Layer, Service};
use tracing::info;

pub async fn start_grpc_server(
    place_params: Option<PlacementCenterServerParams>,
    mqtt_params: Option<MqttBrokerServerParams>,
    journal_params: Option<JournalServerParams>,
    grpc_port: u32,
) -> Result<(), CommonError> {
    let ip = format!("0.0.0.0:{grpc_port}").parse()?;
    let cors_layer = tower_http::cors::CorsLayer::very_permissive();
    let layer = tower::ServiceBuilder::new()
        .layer(BaseMiddlewareLayer::default())
        .into_inner();

    let grpc_max_decoding_message_size = 268435456;
    info!("Broker Grpc Server start success. addr:{}", ip);
    let m_params = place_params.unwrap();
    let b_params = mqtt_params.unwrap();
    let j_params = journal_params.unwrap();
    Server::builder()
        .accept_http1(true)
        .layer(cors_layer)
        .layer(tonic_web::GrpcWebLayer::new())
        .layer(layer)
        .add_service(
            PlacementCenterServiceServer::new(get_place_inner_handler(&m_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            KvServiceServer::new(get_place_kv_handler(&m_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            MqttServiceServer::new(get_place_mqtt_handler(&m_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            EngineServiceServer::new(get_place_engine_handler(&m_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            OpenRaftServiceServer::new(get_place_raft_handler(&m_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            MqttBrokerInnerServiceServer::new(get_mqtt_inner_handler(&b_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            MqttBrokerAdminServiceServer::new(get_mqtt_admin_handler(&b_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            JournalServerAdminServiceServer::new(get_journal_admin_handler(&j_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .add_service(
            JournalServerInnerServiceServer::new(get_journal_inner_handler(&j_params))
                .max_decoding_message_size(grpc_max_decoding_message_size),
        )
        .serve(ip)
        .await?;

    Ok(())
}

fn get_place_inner_handler(place_params: &PlacementCenterServerParams) -> GrpcPlacementService {
    GrpcPlacementService::new(
        place_params.storage_driver.clone(),
        place_params.cache_manager.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.client_pool.clone(),
        place_params.journal_call_manager.clone(),
        place_params.mqtt_call_manager.clone(),
    )
}

fn get_place_kv_handler(place_params: &PlacementCenterServerParams) -> GrpcKvService {
    GrpcKvService::new(
        place_params.storage_driver.clone(),
        place_params.rocksdb_engine_handler.clone(),
    )
}

fn get_place_mqtt_handler(place_params: &PlacementCenterServerParams) -> GrpcMqttService {
    GrpcMqttService::new(
        place_params.cache_manager.clone(),
        place_params.storage_driver.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.mqtt_call_manager.clone(),
        place_params.client_pool.clone(),
    )
}

fn get_place_engine_handler(place_params: &PlacementCenterServerParams) -> GrpcEngineService {
    GrpcEngineService::new(
        place_params.storage_driver.clone(),
        place_params.cache_manager.clone(),
        place_params.rocksdb_engine_handler.clone(),
        place_params.journal_call_manager.clone(),
        place_params.client_pool.clone(),
    )
}

fn get_place_raft_handler(place_params: &PlacementCenterServerParams) -> GrpcOpenRaftServices {
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

fn get_mqtt_admin_handler(mqtt_params: &MqttBrokerServerParams) -> GrpcAdminServices {
    GrpcAdminServices::new(
        mqtt_params.client_pool.clone(),
        mqtt_params.cache_manager.clone(),
        mqtt_params.connection_manager.clone(),
        mqtt_params.subscribe_manager.clone(),
        mqtt_params.metrics_cache_manager.clone(),
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
    use crate::grpc::parse_path;

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
