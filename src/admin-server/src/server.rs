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

use crate::cluster::index;
use crate::cluster::offset::{commit_offset, get_offset_by_group, get_offset_by_timestamp};
use crate::engine::segment::segment_list;
use crate::engine::shard::{shard_create, shard_delete, shard_list};
use crate::mcp::mcp_route;
use crate::{
    cluster::{
        acl::{acl_create, acl_delete, acl_list},
        blacklist::{blacklist_create, blacklist_delete, blacklist_list},
        cluster_config_get, cluster_config_set, cluster_info,
        connector::{connector_create, connector_delete, connector_detail, connector_list},
        health::{health_cluster, health_node, health_ready},
        healthy,
        schema::{
            schema_bind_create, schema_bind_delete, schema_bind_list, schema_create, schema_delete,
            schema_list,
        },
        tenant::{tenant_create, tenant_delete, tenant_list, tenant_update},
        topic::{topic_create, topic_delete, topic_detail, topic_list},
        user::{user_create, user_delete, user_list},
    },
    mqtt::{
        client::client_list,
        monitor::monitor_data,
        overview::overview,
        pub_sub::{read, send},
        session::session_list,
        subscribe::{
            auto_subscribe_create, auto_subscribe_delete, auto_subscribe_list, slow_subscribe_list,
            subscribe_detail, subscribe_list,
        },
        system::{ban_log_list, flapping_detect_list, system_alarm_list},
        topic_rewrite::{topic_rewrite_create, topic_rewrite_delete, topic_rewrite_list},
    },
    nats::mail::mail_list,
    path::*,
    state::HttpState,
};
use axum::routing::get;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use common_metrics::http::record_http_request;
use std::path::PathBuf;
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use tracing::{debug, info, warn};

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
            .merge(mcp_route())
            .with_state(state.clone())
            .layer(middleware::from_fn_with_state(state, rate_limit_middleware))
            .layer(middleware::from_fn(base_middleware))
            .layer(CorsLayer::permissive());

        let listener = tokio::net::TcpListener::bind(&ip).await.unwrap();
        info!(
            "Admin HTTP Server started successfully, listening port: {}, access logging and CORS enabled",
            port
        );

        if let Err(e) = axum::serve(
            listener,
            route.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            panic!("{}", e.to_string());
        }
    }

    fn static_route(&self) -> Router<Arc<HttpState>> {
        let static_dir = PathBuf::from("./dist");

        Router::new().fallback_service(
            ServeDir::new(static_dir).not_found_service(ServeFile::new("./dist/index.html")),
        )
    }

    fn api_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            .merge(self.common_route())
            .merge(self.cluster_resource_route())
            .merge(self.mqtt_route())
            .merge(self.mq9_route())
            .merge(self.kafka_route())
            .merge(self.engine_route())
    }

    fn common_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            .route(STATUS_PATH, get(cluster_info))
            .route(CLUSTER_HEALTHY_PATH, get(healthy))
            .route(HEALTH_READY_PATH, get(health_ready))
            .route(HEALTH_NODE_PATH, get(health_node))
            .route(HEALTH_CLUSTER_PATH, get(health_cluster))
            // config
            .route(CLUSTER_CONFIG_SET_PATH, post(cluster_config_set))
            .route(CLUSTER_CONFIG_GET_PATH, get(cluster_config_get))
            // tenant
            .route(TENANT_LIST_PATH, get(tenant_list))
            .route(TENANT_CREATE_PATH, post(tenant_create))
            .route(TENANT_UPDATE_PATH, post(tenant_update))
            .route(TENANT_DELETE_PATH, post(tenant_delete))
            .route("/", get(index))
    }

    fn engine_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            // shard
            .route(STORAGE_ENGINE_SHARD_LIST_PATH, post(shard_list))
            .route(STORAGE_ENGINE_SHARD_CREATE_PATH, post(shard_create))
            .route(STORAGE_ENGINE_SHARD_DELETE_PATH, post(shard_delete))
            // segment
            .route(STORAGE_ENGINE_SEGMENT_LIST_PATH, post(segment_list))
    }

    fn cluster_resource_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            // topic
            .route(CLUSTER_TOPIC_LIST_PATH, get(topic_list))
            .route(CLUSTER_TOPIC_DETAIL_PATH, get(topic_detail))
            .route(CLUSTER_TOPIC_CREATE_PATH, post(topic_create))
            .route(CLUSTER_TOPIC_DELETE_PATH, post(topic_delete))
            // topic-rewrite
            .route(CLUSTER_TOPIC_REWRITE_LIST_PATH, get(topic_rewrite_list))
            .route(
                CLUSTER_TOPIC_REWRITE_CREATE_PATH,
                post(topic_rewrite_create),
            )
            .route(
                CLUSTER_TOPIC_REWRITE_DELETE_PATH,
                post(topic_rewrite_delete),
            )
            // acl
            .route(CLUSTER_ACL_LIST_PATH, get(acl_list))
            .route(CLUSTER_ACL_CREATE_PATH, post(acl_create))
            .route(CLUSTER_ACL_DELETE_PATH, post(acl_delete))
            // blacklist
            .route(CLUSTER_BLACKLIST_LIST_PATH, get(blacklist_list))
            .route(CLUSTER_BLACKLIST_CREATE_PATH, post(blacklist_create))
            .route(CLUSTER_BLACKLIST_DELETE_PATH, post(blacklist_delete))
            // connector
            .route(CLUSTER_CONNECTOR_LIST_PATH, get(connector_list))
            .route(CLUSTER_CONNECTOR_CREATE_PATH, post(connector_create))
            .route(CLUSTER_CONNECTOR_DETAIL_PATH, get(connector_detail))
            .route(CLUSTER_CONNECTOR_DELETE_PATH, post(connector_delete))
            // schema
            .route(CLUSTER_SCHEMA_LIST_PATH, get(schema_list))
            .route(CLUSTER_SCHEMA_CREATE_PATH, post(schema_create))
            .route(CLUSTER_SCHEMA_DELETE_PATH, post(schema_delete))
            .route(CLUSTER_SCHEMA_BIND_LIST_PATH, get(schema_bind_list))
            .route(CLUSTER_SCHEMA_BIND_CREATE_PATH, post(schema_bind_create))
            .route(CLUSTER_SCHEMA_BIND_DELETE_PATH, post(schema_bind_delete))
            // user
            .route(CLUSTER_USER_LIST_PATH, get(user_list))
            .route(CLUSTER_USER_CREATE_PATH, post(user_create))
            .route(CLUSTER_USER_DELETE_PATH, post(user_delete))
            // offset
            .route(
                CLUSTER_OFFSET_BY_TIMESTAMP_PATH,
                post(get_offset_by_timestamp),
            )
            .route(CLUSTER_OFFSET_BY_GROUP_PATH, post(get_offset_by_group))
            .route(CLUSTER_OFFSET_COMMIT_PATH, post(commit_offset))
    }

    fn mqtt_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
            // overview
            .route(MQTT_OVERVIEW_PATH, get(overview))
            // monitor
            .route(MQTT_MONITOR_PATH, get(monitor_data))
            // client
            .route(MQTT_CLIENT_LIST_PATH, get(client_list))
            // session
            .route(MQTT_SESSION_LIST_PATH, get(session_list))
            // subscribe
            .route(MQTT_SUBSCRIBE_LIST_PATH, get(subscribe_list))
            .route(MQTT_SUBSCRIBE_DETAIL_PATH, get(subscribe_detail))
            // auto subscribe
            .route(MQTT_AUTO_SUBSCRIBE_LIST_PATH, get(auto_subscribe_list))
            .route(MQTT_AUTO_SUBSCRIBE_CREATE_PATH, post(auto_subscribe_create))
            .route(MQTT_AUTO_SUBSCRIBE_DELETE_PATH, post(auto_subscribe_delete))
            // slow subscribe
            .route(MQTT_SLOW_SUBSCRIBE_LIST_PATH, get(slow_subscribe_list))
            // flapping_detect
            .route(MQTT_FLAPPING_DETECT_LIST_PATH, get(flapping_detect_list))
            // system alarm
            .route(MQTT_SYSTEM_ALARM_LIST_PATH, get(system_alarm_list))
            .route(MQTT_BAN_LOG_LIST_PATH, get(ban_log_list))
            // message
            .route(MQTT_MESSAGE_SEND_PATH, post(send))
            .route(MQTT_MESSAGE_READ_PATH, post(read))
    }

    fn mq9_route(&self) -> Router<Arc<HttpState>> {
        Router::new().route(MQ9_MAIL_LIST_PATH, get(mail_list))
    }

    fn kafka_route(&self) -> Router<Arc<HttpState>> {
        Router::new()
    }
}

async fn rate_limit_middleware(
    State(state): State<Arc<HttpState>>,
    uri: Uri,
    request: Request,
    next: Next,
) -> Response {
    if let Err(e) = state
        .rate_limiter
        .http_uri_rate_limit(uri.path().to_string())
        .await
    {
        warn!("HTTP rate limit exceeded for {}: {}", uri.path(), e);
        return (StatusCode::TOO_MANY_REQUESTS, e.to_string()).into_response();
    }
    next.run(request).await
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

    // Process the request
    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();

    // Record HTTP metrics (request count + duration, per method/uri)
    record_http_request(
        method.as_str(),
        &normalize_uri_path(uri.path()),
        status.as_u16(),
        duration.as_millis() as f64,
    );

    // Structured logging with appropriate log levels
    let duration_ms = duration.as_millis();

    // Log with different levels based on status and performance
    match status.as_u16() {
        200..=299 => {
            if duration_ms > 1000 {
                info!(
                    "SLOW REQUEST: {} {} {} - {} - \"{}\" \"{}\" {}ms",
                    method,
                    uri,
                    status.as_u16(),
                    client_ip,
                    user_agent,
                    referer,
                    duration_ms
                );
            } else {
                debug!(
                    "SUCCESS: {} {} {} - {} - \"{}\" \"{}\" {}ms",
                    method,
                    uri,
                    status.as_u16(),
                    client_ip,
                    user_agent,
                    referer,
                    duration_ms
                );
            }
        }
        400..=499 => {
            warn!(
                "CLIENT_ERROR: {} {} {} - {} - \"{}\" \"{}\" {}ms",
                method,
                uri,
                status.as_u16(),
                client_ip,
                user_agent,
                referer,
                duration_ms
            );
        }
        500..=599 => {
            warn!(
                "SERVER_ERROR: {} {} {} - {} - \"{}\" \"{}\" {}ms",
                method,
                uri,
                status.as_u16(),
                client_ip,
                user_agent,
                referer,
                duration_ms
            );
        }
        _ => {
            debug!(
                "OTHER: {} {} {} - {} - \"{}\" \"{}\" {}ms",
                method,
                uri,
                status.as_u16(),
                client_ip,
                user_agent,
                referer,
                duration_ms
            );
        }
    }

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
