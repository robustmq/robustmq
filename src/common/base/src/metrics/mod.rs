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

pub mod broker;
use axum::routing::{get, Route};
use axum::Router;
use prometheus::IntGaugeVec;
use prometheus::{Encoder, TextEncoder};

use crate::log::info;

lazy_static::lazy_static! {
    static ref APP_VERSION: IntGaugeVec =
        prometheus::register_int_gauge_vec!("app_version", "app version", &["short_version", "version"]).unwrap();
}

const SERVER_LABLE_MQTT: &str = "mqtt4";
const SERVER_LABLE_GRPC: &str = "grpc";
const SERVER_LABLE_HTTP: &str = "http";

pub fn dump_metrics() -> String {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let mf = prometheus::gather();
    encoder.encode(&mf, &mut buffer).unwrap();
    let res = String::from_utf8(buffer).unwrap();
    return res;
}

pub async fn register_prometheus_export(port: u16) {
    let ip = format!("0.0.0.0:{}", port);
    let route = Router::new().route("/metrics", get(route_metrics));
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info(&format!(
        "Prometheus HTTP Server started successfully, listening port: {}",
        port
    ));
    axum::serve(listener, route).await.unwrap();
}

pub async fn route_metrics() -> String {
    return dump_metrics();
}
