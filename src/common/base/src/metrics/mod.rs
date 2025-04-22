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

use axum::routing::get;
use axum::Router;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::{LazyLock, Mutex, MutexGuard};
use tracing::info;

pub mod broker;
pub mod counter;
pub mod gauge;
pub mod histogram;

static REGISTRY: LazyLock<Mutex<Registry>> = LazyLock::new(|| Mutex::new(Registry::default()));

pub fn metrics_register_default() -> MutexGuard<'static, Registry> {
    REGISTRY.lock().unwrap()
}

const SERVER_LABEL_MQTT: &str = "mqtt4";
const SERVER_LABEL_GRPC: &str = "grpc";
const SERVER_LABEL_HTTP: &str = "http";

pub fn dump_metrics() -> String {
    let mut buffer = String::new();
    let re = metrics_register_default();
    encode(&mut buffer, &re).unwrap();
    buffer
}

pub async fn register_prometheus_export(port: u32) {
    let ip = format!("0.0.0.0:{}", port);
    let route = Router::new().route("/metrics", get(route_metrics));
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info!(
        "Prometheus HTTP Server started successfully, listening port: {}",
        port
    );
    axum::serve(listener, route).await.unwrap();
}

pub async fn route_metrics() -> String {
    dump_metrics()
}
