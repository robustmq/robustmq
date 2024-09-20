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

use super::index::{caches, index, metrics, list_cluster, list_node};
use super::mqtt::mqtt_routes;
use crate::raft::metadata::RaftGroupMetadata;
use crate::{
    cache::{journal::JournalCacheManager, placement::PlacementCacheManager},
    storage::placement::raft::RaftMachineStorage,
};
use axum::routing::get;
use axum::Router;
use common_base::config::placement_center::placement_center_conf;
use log::info;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use super::list_path;

pub const ROUTE_ROOT: &str = "/";
pub const ROUTE_METRICS: &str = "/metrics";
pub const ROUTE_CACHES: &str = "/caches";
pub const ROUTE_CLUSTER: &str = "/cluster";
pub const ROUTE_CLUSTER_NODE: &str = "/cluster/node";

#[derive(Clone)]
#[allow(dead_code)]
pub struct HttpServerState {
    pub raft_metadata: Arc<RwLock<RaftGroupMetadata>>,
    pub raft_storage: Arc<RwLock<RaftMachineStorage>>,
    pub cluster_cache: Arc<PlacementCacheManager>,
    pub engine_cache: Arc<JournalCacheManager>,
}

impl HttpServerState {
    pub fn new(
        placement_cache: Arc<RwLock<RaftGroupMetadata>>,
        raft_storage: Arc<RwLock<RaftMachineStorage>>,
        cluster_cache: Arc<PlacementCacheManager>,
        engine_cache: Arc<JournalCacheManager>,
    ) -> Self {
        return Self {
            raft_metadata: placement_cache,
            raft_storage,
            cluster_cache,
            engine_cache,
        };
    }
}

pub async fn start_http_server(state: HttpServerState) {
    let config = placement_center_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.network.http_port).parse().unwrap();
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
        .route(ROUTE_ROOT, get(index))
        .route(ROUTE_CACHES, get(caches))
        .route(ROUTE_METRICS, get(metrics))
        .route(&list_path(ROUTE_CLUSTER), get(list_cluster))
        .route(&list_path(ROUTE_CLUSTER_NODE), get(list_node));

    let mqtt = mqtt_routes();

    let app = Router::new().merge(common).merge(mqtt);
    return app.with_state(state);
}
