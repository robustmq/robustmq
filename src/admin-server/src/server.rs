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
    cluster::{cluster_config_get, cluster_config_set},
    mqtt::{
        acl::{acl_create, acl_delete, acl_list},
        blacklist::{blacklist_create, blacklist_delete, blacklist_list},
        client::client_list,
        connector::{connector_create, connector_delete, connector_list},
        overview::{overview, overview_metrics},
        schema::{
            schema_bind_create, schema_bind_delete, schema_bind_list, schema_create, schema_delete,
            schema_list,
        },
        session::session_list,
        subscribe::{
            auto_subscribe_create, auto_subscribe_delete, auto_subscribe_list, slow_subscribe_list,
            subscribe_detail, subscribe_list,
        },
        system::{flapping_detect_list, system_alarm_list},
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
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        info!(
            "Admin HTTP Server started successfully, listening port: {}",
            port
        );
        axum::serve(listener, route).await.unwrap();
    }

    fn common_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            .route("/", get(index))
            // config
            .route("/cluster/config/set", post(cluster_config_set))
            .route("/cluster/config/get", post(cluster_config_get))
    }

    fn mqtt_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            // overview
            .route("/mqtt/overview", post(overview))
            .route("/mqtt/overview/metrics", post(overview_metrics))
            // client
            .route("/mqtt/client/list", post(client_list))
            // session
            .route("/mqtt/session/list", post(session_list))
            // topic
            .route("/mqtt/topic/list", post(topic_list))
            // topic-rewrite
            .route("/mqtt/topic-rewrite/list", post(topic_rewrite_list))
            .route("/mqtt/topic-rewrite/create", post(topic_rewrite_create))
            .route("/mqtt/topic-rewrite/delete", post(topic_list))
            // subscribe
            .route("/mqtt/subscribe/list", post(subscribe_list))
            .route("/mqtt/subscribe/detail", post(subscribe_detail))
            // auto subscribe
            .route("/mqtt/auto-subscribe/list", post(auto_subscribe_list))
            .route("/mqtt/auto-subscribe/create", post(auto_subscribe_create))
            .route("/mqtt/auto-subscribe/delete", post(auto_subscribe_delete))
            // slow subscribe
            .route("/mqtt/slow-subscribe/list", post(slow_subscribe_list))
            // user
            .route("/mqtt/user/list", post(user_list))
            .route("/mqtt/user/create", post(user_create))
            .route("/mqtt/user/delete", post(user_delete))
            // acl
            .route("/mqtt/acl/list", post(acl_list))
            .route("/mqtt/acl/create", post(acl_create))
            .route("/mqtt/acl/delete", post(acl_delete))
            // blacklist
            .route("/mqtt/blacklist/list", post(blacklist_list))
            .route("/mqtt/blacklist/create", post(blacklist_create))
            .route("/mqtt/blacklist/delete", post(blacklist_delete))
            // flapping_detect
            .route("/mqtt/flapping_detect/list", post(flapping_detect_list))
            // connector
            .route("/mqtt/connector/list", post(connector_list))
            .route("/mqtt/connector/create", post(connector_create))
            .route("/mqtt/connector/delete", post(connector_delete))
            // schema
            .route("/mqtt/schema/list", post(schema_list))
            .route("/mqtt/schema/create", post(schema_create))
            .route("/mqtt/schema/delete", post(schema_delete))
            .route("/mqtt/schema-bind/list", post(schema_bind_list))
            .route("/mqtt/schema-bind/create", post(schema_bind_create))
            .route("/mqtt/schema-bind/delete", post(schema_bind_delete))
            // system alarm
            .route("/mqtt/system-alarm/list", post(system_alarm_list))
    }

    fn kafka_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
    }
}

pub async fn index(State(_state): State<Arc<HttpState>>) -> String {
    format!("RobustMQ {}", version())
}
