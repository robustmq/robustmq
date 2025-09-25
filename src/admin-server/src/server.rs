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
use common_metrics::http::{
    record_http_connection_end, record_http_connection_start, record_http_request,
};
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
            .layer(middleware::from_fn(base_middleware))
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

/// Enhanced HTTP middleware with comprehensive metrics and logging
async fn base_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Record connection start for active connection tracking
    record_http_connection_start();

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

    // Extract request size from Content-Length header
    let request_size = headers
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok());

    // Process the request
    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();

    // Extract response size from Content-Length header
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok());

    // Record comprehensive HTTP metrics with all dimensions
    record_http_request(
        method.to_string(),
        normalize_uri_path(uri.path()), // Normalize path to reduce cardinality
        status.as_u16(),
        duration.as_millis() as f64,
        request_size,
        response_size,
    );

    // Structured logging with appropriate log levels
    let duration_ms = duration.as_millis();

    // Log with different levels based on status and performance
    match status.as_u16() {
        200..=299 => {
            if duration_ms > 1000 {
                // Slow requests (>1s)
                warn!(
                    "SLOW REQUEST: {} {} {} - {} - \"{}\" \"{}\" {}ms | req_size: {} | resp_size: {}",
                    method, uri, status.as_u16(), client_ip, user_agent, referer, duration_ms,
                    format_size(request_size), format_size(response_size)
                );
            } else {
                info!(
                    "SUCCESS: {} {} {} - {} - \"{}\" \"{}\" {}ms | req_size: {} | resp_size: {}",
                    method,
                    uri,
                    status.as_u16(),
                    client_ip,
                    user_agent,
                    referer,
                    duration_ms,
                    format_size(request_size),
                    format_size(response_size)
                );
            }
        }
        400..=499 => {
            warn!(
                "CLIENT_ERROR: {} {} {} - {} - \"{}\" \"{}\" {}ms | req_size: {} | resp_size: {}",
                method,
                uri,
                status.as_u16(),
                client_ip,
                user_agent,
                referer,
                duration_ms,
                format_size(request_size),
                format_size(response_size)
            );
        }
        500..=599 => {
            warn!(
                "SERVER_ERROR: {} {} {} - {} - \"{}\" \"{}\" {}ms | req_size: {} | resp_size: {}",
                method,
                uri,
                status.as_u16(),
                client_ip,
                user_agent,
                referer,
                duration_ms,
                format_size(request_size),
                format_size(response_size)
            );
        }
        _ => {
            info!(
                "OTHER: {} {} {} - {} - \"{}\" \"{}\" {}ms | req_size: {} | resp_size: {}",
                method,
                uri,
                status.as_u16(),
                client_ip,
                user_agent,
                referer,
                duration_ms,
                format_size(request_size),
                format_size(response_size)
            );
        }
    }

    // Record connection end for active connection tracking
    record_http_connection_end();

    response
}

/// Normalize URI path to reduce metric cardinality
/// Replaces dynamic segments with placeholders
fn normalize_uri_path(path: &str) -> String {
    // Replace common dynamic segments to reduce cardinality
    let normalized = path
        .split('/')
        .map(|segment| {
            // Replace UUIDs, IDs, and other dynamic segments
            if segment.chars().all(|c| c.is_ascii_digit()) && segment.len() > 3 {
                "{id}".to_string()
            } else if segment.len() == 36 && segment.chars().filter(|&c| c == '-').count() == 4 {
                "{uuid}".to_string()
            } else if segment.len() > 20 && segment.chars().all(|c| c.is_ascii_alphanumeric()) {
                "{hash}".to_string()
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    // Limit path length to prevent extremely long paths
    if normalized.len() > 100 {
        format!("{}...", &normalized[..97])
    } else {
        normalized
    }
}

/// Format size for logging (handles None values gracefully)
fn format_size(size: Option<f64>) -> String {
    match size {
        Some(s) if s < 1024.0 => format!("{}B", s as u64),
        Some(s) if s < 1024.0 * 1024.0 => format!("{:.1}KB", s / 1024.0),
        Some(s) if s < 1024.0 * 1024.0 * 1024.0 => format!("{:.1}MB", s / (1024.0 * 1024.0)),
        Some(s) => format!("{:.1}GB", s / (1024.0 * 1024.0 * 1024.0)),
        None => "unknown".to_string(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_uri_path() {
        // Test normal paths
        assert_eq!(normalize_uri_path("/api/users"), "/api/users");
        assert_eq!(normalize_uri_path("/api/v1/health"), "/api/v1/health");

        // Test paths with IDs
        assert_eq!(normalize_uri_path("/api/users/12345"), "/api/users/{id}");
        assert_eq!(
            normalize_uri_path("/api/orders/67890/items"),
            "/api/orders/{id}/items"
        );

        // Test paths with UUIDs
        assert_eq!(
            normalize_uri_path("/api/users/550e8400-e29b-41d4-a716-446655440000"),
            "/api/users/{uuid}"
        );

        // Test paths with hashes
        assert_eq!(
            normalize_uri_path("/api/files/abcdef1234567890abcdef1234567890"),
            "/api/files/{hash}"
        );

        // Test very long paths
        let long_path = "/api/".to_string() + &"a".repeat(200);
        let normalized = normalize_uri_path(&long_path);
        assert!(normalized.len() <= 100);
        if normalized.len() == 100 {
            assert!(normalized.ends_with("..."));
        }
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(None), "unknown");
        assert_eq!(format_size(Some(512.0)), "512B");
        assert_eq!(format_size(Some(1536.0)), "1.5KB");
        assert_eq!(format_size(Some(2097152.0)), "2.0MB");
        assert_eq!(format_size(Some(1073741824.0)), "1.0GB");
    }

    #[test]
    fn test_extract_client_ip() {
        use axum::http::HeaderMap;
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        // Test with no headers
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers, socket_addr), "127.0.0.1");

        // Test with X-Forwarded-For header
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "192.168.1.100, 10.0.0.1".parse().unwrap(),
        );
        assert_eq!(extract_client_ip(&headers, socket_addr), "192.168.1.100");

        // Test with X-Real-IP header
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "192.168.1.200".parse().unwrap());
        assert_eq!(extract_client_ip(&headers, socket_addr), "192.168.1.200");
    }
}
