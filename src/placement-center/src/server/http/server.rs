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

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use common_base::config::placement_center::placement_center_conf;
use log::info;

use super::index::{metrics, raft_metrics};
use crate::core::cache::PlacementCacheManager;
use crate::journal::cache::JournalCacheManager;
use crate::route::apply::RaftMachineApply;

pub const ROUTE_METRICS: &str = "/metrics";
pub const RAFT_METRICS: &str = "/raft_metrics";

#[derive(Clone)]
#[allow(dead_code)]
pub struct HttpServerState {
    pub placement_cache: Arc<PlacementCacheManager>,
    pub cluster_cache: Arc<PlacementCacheManager>,
    pub engine_cache: Arc<JournalCacheManager>,
    pub placement_center_storage: Arc<RaftMachineApply>,
}

impl HttpServerState {
    pub fn new(
        placement_cache: Arc<PlacementCacheManager>,
        cluster_cache: Arc<PlacementCacheManager>,
        engine_cache: Arc<JournalCacheManager>,
        placement_center_storage: Arc<RaftMachineApply>,
    ) -> Self {
        Self {
            placement_cache,
            cluster_cache,
            engine_cache,
            placement_center_storage,
        }
    }
}

pub async fn start_http_server(state: HttpServerState) {
    let config = placement_center_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.network.http_port)
        .parse()
        .unwrap();
    let app = routes(state);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info!(
        "Placement Center HTTP Server start success. bind addr:{}",
        ip
    );
    axum::serve(listener, app).await.unwrap();
}

fn routes(state: HttpServerState) -> Router {
    let common = Router::new()
        .route(ROUTE_METRICS, get(metrics))
        .route(RAFT_METRICS, get(raft_metrics));

    let app = Router::new().merge(common);
    app.with_state(state)
}
