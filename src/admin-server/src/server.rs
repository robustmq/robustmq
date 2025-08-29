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

use axum::{routing::get, Router};
use tracing::info;

use crate::mqtt::session::session_list;

pub struct DashboardServer {}

impl Default for DashboardServer {
    fn default() -> Self {
        Self::new()
    }
}
impl DashboardServer {
    pub fn new() -> Self {
        DashboardServer {}
    }

    pub async fn start(port: u32) {
        let ip = format!("0.0.0.0:{port}");
        let route = Router::new().route("/session/list", get(session_list));
        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        info!(
            "Prometheus HTTP Server started successfully, listening port: {}",
            port
        );
        axum::serve(listener, route).await.unwrap();
    }
}
