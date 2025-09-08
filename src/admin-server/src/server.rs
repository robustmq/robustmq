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
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, Method, Uri},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use common_base::version::version;
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tracing::{info, warn};

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
            .with_state(state)
            .layer(middleware::from_fn(access_log_middleware));

        let listener = tokio::net::TcpListener::bind(&ip).await.unwrap();
        info!(
            "Admin HTTP Server started successfully, listening port: {}, access logging enabled",
            port
        );
        axum::serve(
            listener,
            route.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
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

async fn access_log_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let client_ip = extract_client_ip(&headers, addr);
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    let referer = headers
        .get("referer")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("-");
    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();
    info!(
        "http request Log: {} {} {} - {} - \"{}\" \"{}\" {}ms",
        method,
        uri,
        status.as_u16(),
        client_ip,
        user_agent,
        referer,
        duration.as_millis()
    );

    if status.is_client_error() || status.is_server_error() {
        warn!(
            "HTTP {} {} {} - {} - FAILED",
            method,
            uri,
            status.as_u16(),
            client_ip
        );
    }

    response
}

fn extract_client_ip(headers: &HeaderMap, socket_addr: SocketAddr) -> String {
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return real_ip_str.to_string();
        }
    }

    if let Some(forwarded) = headers.get("forwarded") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            for part in forwarded_str.split(';') {
                if let Some(for_part) = part.trim().strip_prefix("for=") {
                    return for_part.to_string();
                }
            }
        }
    }
    socket_addr.ip().to_string()
}

pub async fn index(State(_state): State<Arc<HttpState>>) -> String {
    format!("RobustMQ {}", version())
}
