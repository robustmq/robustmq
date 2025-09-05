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

use crate::{
    mqtt::{
        acl::acl_list,
        blacklist::blacklist_list,
        client::client_list,
        connector::connector_list,
        overview::{overview, overview_metrics},
        schema::schema_list,
        session::session_list,
        subscribe::subscribe_list,
        topic::topic_list,
        user::user_list,
    },
    state::HttpState,
};
use axum::{extract::State, routing::get, Router};
use common_base::version::version;
use std::sync::Arc;
use tracing::info;

pub struct AdminServer {}

impl Default for AdminServer {
    fn default() -> Self {
        Self::new()
    }
}
impl AdminServer {
    pub fn new() -> Self {
        AdminServer {}
    }

    pub async fn start(&self, port: u32, state: Arc<HttpState>) {
        let ip = format!("0.0.0.0:{port}");
        let route = Router::new()
            .merge(self.common_route())
            .merge(self.mqtt_route())
            .merge(self.kafka_route())
            .merge(self.meta_route())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        info!(
            "Admin HTTP Server started successfully, listening port: {}",
            port
        );
        axum::serve(listener, route).await.unwrap();
    }

    fn common_route(&self) -> Router<Arc<HttpState>> {
        Router::new().route("/", get(index))
    }

    fn mqtt_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            .route("/mqtt/overview", get(overview))
            .route("/mqtt/overview-metrics", get(overview_metrics))
            .route("/mqtt/client-list", get(client_list))
            .route("/mqtt/session-list", get(session_list))
            .route("/mqtt/topic-list", get(topic_list))
            .route("/mqtt/subscribe-list", get(subscribe_list))
            .route("/mqtt/user-list", get(user_list))
            .route("/mqtt/acl-list", get(acl_list))
            .route("/mqtt/blacklist-list", get(blacklist_list))
            .route("/mqtt/connector-list", get(connector_list))
            .route("/mqtt/schema-list", get(schema_list))
    }

    fn kafka_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
    }

    fn meta_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
    }
}

pub async fn index(State(_state): State<Arc<HttpState>>) -> String {
    format!("RobustMQ {}", version())
}
