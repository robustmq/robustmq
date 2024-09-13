// Copyright 2023 RobustMQ Team
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

use super::{
    cache::{cache_info, index, metrics},
    publish::{connection_list, http_publish},
};
use crate::handler::cache::CacheManager;
use crate::subscribe::subscribe_manager::SubscribeManager;
use axum::routing::get;
use axum::Router;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use log::info;
use std::{net::SocketAddr, sync::Arc};

pub const ROUTE_ROOT: &str = "/";
pub const ROUTE_PUBLISTH: &str = "/publish";
pub const ROUTE_CONNECTION: &str = "/connection";
pub const ROUTE_CACHE: &str = "/caches";
pub const ROUTE_METRICS: &str = "/metrics";

#[derive(Clone)]
pub struct HttpServerState {
    pub cache_metadata: Arc<CacheManager>,
    pub subscribe_cache: Arc<SubscribeManager>,
}

impl HttpServerState {
    pub fn new(cache_metadata: Arc<CacheManager>, subscribe_cache: Arc<SubscribeManager>) -> Self {
        return Self {
            cache_metadata,
            subscribe_cache,
        };
    }
}

pub async fn start_http_server(state: HttpServerState) {
    let config = broker_mqtt_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.http_port).parse().unwrap();
    let app = routes_v1(state);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info!(
        "Broker HTTP Server start success. bind addr:{}",
        config.http_port
    );
    axum::serve(listener, app).await.unwrap();
}

fn routes_v1(state: HttpServerState) -> Router {
    let meta = Router::new()
        .route(ROUTE_ROOT, get(index))
        .route(ROUTE_CACHE, get(cache_info))
        .route(ROUTE_PUBLISTH, get(http_publish))
        .route(ROUTE_CONNECTION, get(connection_list))
        .route(ROUTE_METRICS, get(metrics));

    let app = Router::new().merge(meta);
    return app.with_state(state);
}
