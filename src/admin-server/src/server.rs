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
        system::{ban_log_list, flapping_detect_list, system_alarm_list},
        topic::{topic_list, topic_rewrite_create, topic_rewrite_list},
        user::{user_create, user_delete, user_list},
    },
    path::*,
    state::HttpState,
};
use axum::response::Html;
use axum::response::IntoResponse;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, Method, Uri},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use common_base::version::version;
use reqwest::StatusCode;
use std::path::PathBuf;
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tokio::fs;
use tower_http::{cors::CorsLayer, services::ServeDir};
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
            .merge(self.static_route())
            .nest("/api", self.api_route())
            .with_state(state)
            .layer(middleware::from_fn(access_log_middleware))
            .layer(CorsLayer::permissive());

        let listener = tokio::net::TcpListener::bind(&ip).await.unwrap();
        info!(
            "Admin HTTP Server started successfully, listening port: {}, access logging and CORS enabled",
            port
        );
        axum::serve(
            listener,
            route.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    }

    fn static_route(&self) -> Router<Arc<HttpState>> {
        let static_dir = PathBuf::from("./dist");

        Router::new()
            .nest_service("/", ServeDir::new(static_dir))
            .fallback(serve_spa_fallback)
    }

    fn api_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            .merge(self.common_route())
            .merge(self.mqtt_route())
            .merge(self.kafka_route())
    }

    fn common_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            .route(STATUS_PATH, get(index))
            // config
            .route(CLUSTER_CONFIG_SET_PATH, post(cluster_config_set))
            .route(CLUSTER_CONFIG_GET_PATH, post(cluster_config_get))
    }

    fn mqtt_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            // overview
            .route(MQTT_OVERVIEW_PATH, post(overview))
            .route(MQTT_OVERVIEW_METRICS_PATH, post(overview_metrics))
            // client
            .route(MQTT_CLIENT_LIST_PATH, post(client_list))
            // session
            .route(MQTT_SESSION_LIST_PATH, post(session_list))
            // topic
            .route(MQTT_TOPIC_LIST_PATH, post(topic_list))
            // topic-rewrite
            .route(MQTT_TOPIC_REWRITE_LIST_PATH, post(topic_rewrite_list))
            .route(MQTT_TOPIC_REWRITE_CREATE_PATH, post(topic_rewrite_create))
            .route(MQTT_TOPIC_REWRITE_DELETE_PATH, post(topic_list))
            // subscribe
            .route(MQTT_SUBSCRIBE_LIST_PATH, post(subscribe_list))
            .route(MQTT_SUBSCRIBE_DETAIL_PATH, post(subscribe_detail))
            // auto subscribe
            .route(MQTT_AUTO_SUBSCRIBE_LIST_PATH, post(auto_subscribe_list))
            .route(MQTT_AUTO_SUBSCRIBE_CREATE_PATH, post(auto_subscribe_create))
            .route(MQTT_AUTO_SUBSCRIBE_DELETE_PATH, post(auto_subscribe_delete))
            // slow subscribe
            .route(MQTT_SLOW_SUBSCRIBE_LIST_PATH, post(slow_subscribe_list))
            // user
            .route(MQTT_USER_LIST_PATH, post(user_list))
            .route(MQTT_USER_CREATE_PATH, post(user_create))
            .route(MQTT_USER_DELETE_PATH, post(user_delete))
            // acl
            .route(MQTT_ACL_LIST_PATH, post(acl_list))
            .route(MQTT_ACL_CREATE_PATH, post(acl_create))
            .route(MQTT_ACL_DELETE_PATH, post(acl_delete))
            // blacklist
            .route(MQTT_BLACKLIST_LIST_PATH, post(blacklist_list))
            .route(MQTT_BLACKLIST_CREATE_PATH, post(blacklist_create))
            .route(MQTT_BLACKLIST_DELETE_PATH, post(blacklist_delete))
            // flapping_detect
            .route(MQTT_FLAPPING_DETECT_LIST_PATH, post(flapping_detect_list))
            // connector
            .route(MQTT_CONNECTOR_LIST_PATH, post(connector_list))
            .route(MQTT_CONNECTOR_CREATE_PATH, post(connector_create))
            .route(MQTT_CONNECTOR_DELETE_PATH, post(connector_delete))
            // schema
            .route(MQTT_SCHEMA_LIST_PATH, post(schema_list))
            .route(MQTT_SCHEMA_CREATE_PATH, post(schema_create))
            .route(MQTT_SCHEMA_DELETE_PATH, post(schema_delete))
            .route(MQTT_SCHEMA_BIND_LIST_PATH, post(schema_bind_list))
            .route(MQTT_SCHEMA_BIND_CREATE_PATH, post(schema_bind_create))
            .route(MQTT_SCHEMA_BIND_DELETE_PATH, post(schema_bind_delete))
            // system alarm
            .route(MQTT_SYSTEM_ALARM_LIST_PATH, post(system_alarm_list))
            .route(MQTT_BAN_LOG_LIST_PATH, post(ban_log_list))
    }

    fn kafka_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
    }
}

async fn serve_spa_fallback() -> impl IntoResponse {
    let index_path = PathBuf::from("index.html");

    match fs::read_to_string(&index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "404 - Page not found. Missing the admin dashboard page",
        )
            .into_response(),
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
        "http request log: {} {} {} - {} - \"{}\" \"{}\" {}ms",
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
            "http request log: {} {} {} - {} - FAILED",
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
    format!("RobustMQ API {}", version())
}
