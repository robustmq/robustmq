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

use super::index::metrics;
use super::raft::{add_leadrner, change_membership, init, raft_status};
use super::v1_path;
use crate::cache::{journal::JournalCacheManager, placement::PlacementCacheManager};
use crate::raftv1::rocksdb::RaftMachineStorage;
use crate::storage::route::apply::RaftMachineApply;
use axum::routing::{get, post};
use axum::Router;
use common_base::config::placement_center::placement_center_conf;
use log::info;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

pub const ROUTE_METRICS: &str = "/metrics";
pub const ROUTE_CLUSTER_ADD_LEARNER: &str = "/cluster/add-learner";
pub const ROUTE_CLUSTER_CHANGE_MEMBERSHIP: &str = "/cluster/change-membership";
pub const ROUTE_CLUSTER_INIT: &str = "/cluster/init";
pub const ROUTE_CLUSTER_STATUS: &str = "/cluster/status";

#[derive(Clone)]
#[allow(dead_code)]
pub struct HttpServerState {
    pub placement_cache: Arc<PlacementCacheManager>,
    pub raft_storage: Arc<RwLock<RaftMachineStorage>>,
    pub cluster_cache: Arc<PlacementCacheManager>,
    pub engine_cache: Arc<JournalCacheManager>,
    pub placement_center_storage: Arc<RaftMachineApply>,
}

impl HttpServerState {
    pub fn new(
        placement_cache: Arc<PlacementCacheManager>,
        raft_storage: Arc<RwLock<RaftMachineStorage>>,
        cluster_cache: Arc<PlacementCacheManager>,
        engine_cache: Arc<JournalCacheManager>,
        placement_center_storage: Arc<RaftMachineApply>,
    ) -> Self {
        return Self {
            placement_cache,
            raft_storage,
            cluster_cache,
            engine_cache,
            placement_center_storage,
        };
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
        .route(&&v1_path(ROUTE_CLUSTER_ADD_LEARNER), post(add_leadrner))
        .route(
            &v1_path(ROUTE_CLUSTER_CHANGE_MEMBERSHIP),
            post(change_membership),
        )
        .route(&v1_path(ROUTE_CLUSTER_INIT), post(init))
        .route(&v1_path(ROUTE_CLUSTER_STATUS), get(raft_status));

    let app = Router::new().merge(common);
    return app.with_state(state);
}
