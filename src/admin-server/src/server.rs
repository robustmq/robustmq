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
        schema::{
            schema_bind_create, schema_bind_delete, schema_bind_list, schema_create, schema_delete,
            schema_list,
        },
        session::session_list,
        subscribe::{
            auto_subscribe_create, auto_subscribe_delete, auto_subscribe_list, subscribe_list,
        },
        topic::{topic_list, topic_rewrite_create, topic_rewrite_list},
        user::{user_create, user_delete, user_list},
    },
    state::HttpState,
};
use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
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
            .route("/mqtt/overview/metrics", get(overview_metrics))
            // client
            .route("/mqtt/client/list", get(client_list))
            // session
            .route("/mqtt/session/list", get(session_list))
            // topic
            .route("/mqtt/topic/list", get(topic_list))
            // topic-rewrite
            .route("/mqtt/topic-rewrite/list", get(topic_rewrite_list))
            .route("/mqtt/topic-rewrite/create", get(topic_rewrite_create))
            .route("/mqtt/topic-rewrite/delete", get(topic_list))
            // subscribe
            .route("/mqtt/subscribe/list", get(subscribe_list))
            .route("/mqtt/subscribe/detail", get(subscribe_list))
            // auto subscribe
            .route("/mqtt/auto-subscribe/list", get(auto_subscribe_list))
            .route("/mqtt/auto-subscribe/create", get(auto_subscribe_create))
            .route("/mqtt/auto-subscribe/delete", get(auto_subscribe_delete))
            // user
            .route("/mqtt/user/list", get(user_list))
            .route("/mqtt/user/create", post(user_create))
            .route("/mqtt/user/delete", post(user_delete))
            // acl
            .route("/mqtt/acl/list", get(acl_list))
            // blacklist
            .route("/mqtt/blacklist/list", get(blacklist_list))
            // connector
            .route("/mqtt/connector/list", get(connector_list))
            // schema
            .route("/mqtt/schema/list", get(schema_list))
            .route("/mqtt/schema/create", get(schema_create))
            .route("/mqtt/schema/delete", get(schema_delete))
            .route("/mqtt/schema-bind/list", get(schema_bind_list))
            .route("/mqtt/schema-bind/create", get(schema_bind_create))
            .route("/mqtt/schema-bind/delete", get(schema_bind_delete))
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
