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

//! API path constants for RobustMQ Admin Server
//!
//! This module centralizes all API path definitions to avoid duplication
//! between client.rs and server.rs. When adding new endpoints, define
//! the path constant here and use it in both client and server code.

// Common API paths
pub const STATUS_PATH: &str = "/status";
pub const HEALTH_READY_PATH: &str = "/health/ready";
pub const HEALTH_NODE_PATH: &str = "/health/node";
pub const HEALTH_CLUSTER_PATH: &str = "/health/cluster";

// ── /cluster ─────────────────────────────────────────────────────────────────

// Cluster base
pub const CLUSTER_CONFIG_SET_PATH: &str = "/cluster/config/set";
pub const CLUSTER_CONFIG_GET_PATH: &str = "/cluster/config/get";
pub const CLUSTER_HEALTHY_PATH: &str = "/cluster/healthy";

// Cluster Topic API paths
pub const CLUSTER_TOPIC_LIST_PATH: &str = "/cluster/topic/list";
pub const CLUSTER_TOPIC_DETAIL_PATH: &str = "/cluster/topic/detail";
pub const CLUSTER_TOPIC_DELETE_PATH: &str = "/cluster/topic/delete";

// Cluster Topic Rewrite API paths
pub const CLUSTER_TOPIC_REWRITE_LIST_PATH: &str = "/cluster/topic-rewrite/list";
pub const CLUSTER_TOPIC_REWRITE_CREATE_PATH: &str = "/cluster/topic-rewrite/create";
pub const CLUSTER_TOPIC_REWRITE_DELETE_PATH: &str = "/cluster/topic-rewrite/delete";

// Cluster ACL API paths
pub const CLUSTER_ACL_LIST_PATH: &str = "/cluster/acl/list";
pub const CLUSTER_ACL_CREATE_PATH: &str = "/cluster/acl/create";
pub const CLUSTER_ACL_DELETE_PATH: &str = "/cluster/acl/delete";

// Cluster Blacklist API paths
pub const CLUSTER_BLACKLIST_LIST_PATH: &str = "/cluster/blacklist/list";
pub const CLUSTER_BLACKLIST_CREATE_PATH: &str = "/cluster/blacklist/create";
pub const CLUSTER_BLACKLIST_DELETE_PATH: &str = "/cluster/blacklist/delete";

// Cluster Connector API paths
pub const CLUSTER_CONNECTOR_LIST_PATH: &str = "/cluster/connector/list";
pub const CLUSTER_CONNECTOR_CREATE_PATH: &str = "/cluster/connector/create";
pub const CLUSTER_CONNECTOR_DETAIL_PATH: &str = "/cluster/connector/detail";
pub const CLUSTER_CONNECTOR_DELETE_PATH: &str = "/cluster/connector/delete";

// Cluster Schema API paths
pub const CLUSTER_SCHEMA_LIST_PATH: &str = "/cluster/schema/list";
pub const CLUSTER_SCHEMA_CREATE_PATH: &str = "/cluster/schema/create";
pub const CLUSTER_SCHEMA_DELETE_PATH: &str = "/cluster/schema/delete";
pub const CLUSTER_SCHEMA_BIND_LIST_PATH: &str = "/cluster/schema-bind/list";
pub const CLUSTER_SCHEMA_BIND_CREATE_PATH: &str = "/cluster/schema-bind/create";
pub const CLUSTER_SCHEMA_BIND_DELETE_PATH: &str = "/cluster/schema-bind/delete";

// Cluster User API paths
pub const CLUSTER_USER_LIST_PATH: &str = "/cluster/user/list";
pub const CLUSTER_USER_CREATE_PATH: &str = "/cluster/user/create";
pub const CLUSTER_USER_DELETE_PATH: &str = "/cluster/user/delete";

// ── /mqtt ─────────────────────────────────────────────────────────────────────

// MQTT Overview
pub const MQTT_OVERVIEW_PATH: &str = "/mqtt/overview";
pub const MQTT_MONITOR_PATH: &str = "/mqtt/monitor/data";

// MQTT Client
pub const MQTT_CLIENT_LIST_PATH: &str = "/mqtt/client/list";

// MQTT Session
pub const MQTT_SESSION_LIST_PATH: &str = "/mqtt/session/list";

// MQTT Subscribe
pub const MQTT_SUBSCRIBE_LIST_PATH: &str = "/mqtt/subscribe/list";
pub const MQTT_SUBSCRIBE_DETAIL_PATH: &str = "/mqtt/subscribe/detail";

// MQTT Auto Subscribe
pub const MQTT_AUTO_SUBSCRIBE_LIST_PATH: &str = "/mqtt/auto-subscribe/list";
pub const MQTT_AUTO_SUBSCRIBE_CREATE_PATH: &str = "/mqtt/auto-subscribe/create";
pub const MQTT_AUTO_SUBSCRIBE_DELETE_PATH: &str = "/mqtt/auto-subscribe/delete";

// MQTT Slow Subscribe
pub const MQTT_SLOW_SUBSCRIBE_LIST_PATH: &str = "/mqtt/slow-subscribe/list";

// MQTT Flapping Detect
pub const MQTT_FLAPPING_DETECT_LIST_PATH: &str = "/mqtt/flapping_detect/list";

// MQTT System
pub const MQTT_SYSTEM_ALARM_LIST_PATH: &str = "/mqtt/system-alarm/list";
pub const MQTT_BAN_LOG_LIST_PATH: &str = "/mqtt/ban-log/list";

// MQTT Message
pub const MQTT_MESSAGE_SEND_PATH: &str = "/mqtt/message/send";
pub const MQTT_MESSAGE_READ_PATH: &str = "/mqtt/message/read";

// ── /storage-engine ───────────────────────────────────────────────────────────

pub const STORAGE_ENGINE_SHARD_LIST_PATH: &str = "/storage-engine/shard/list";
pub const STORAGE_ENGINE_SHARD_CREATE_PATH: &str = "/storage-engine/shard/create";
pub const STORAGE_ENGINE_SHARD_DELETE_PATH: &str = "/storage-engine/shard/delete";
pub const STORAGE_ENGINE_SEGMENT_LIST_PATH: &str = "/storage-engine/segment/list";

// Cluster Offset API paths
pub const CLUSTER_OFFSET_BY_TIMESTAMP_PATH: &str = "/cluster/offset/timestamp";
pub const CLUSTER_OFFSET_BY_GROUP_PATH: &str = "/cluster/offset/group";
pub const CLUSTER_OFFSET_COMMIT_PATH: &str = "/cluster/offset/commit";

// Cluster Tenant (full CRUD, lives in cluster/tenant.rs)
pub const TENANT_LIST_PATH: &str = "/cluster/tenant/list";
pub const TENANT_CREATE_PATH: &str = "/cluster/tenant/create";
pub const TENANT_UPDATE_PATH: &str = "/cluster/tenant/update";
pub const TENANT_DELETE_PATH: &str = "/cluster/tenant/delete";

// ── MCP ───────────────────────────────────────────────────────────────────────

// MCP Server path (outside /api prefix — mounted directly on root)
pub const MCP_PATH: &str = "/mcp";

// ── Utilities ─────────────────────────────────────────────────────────────────

pub const API_PREFIX: &str = "/api";

/// Build full API path with prefix
pub fn api_path(path: &str) -> String {
    format!("{API_PREFIX}{path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_path() {
        assert_eq!(api_path(MQTT_OVERVIEW_PATH), "/api/mqtt/overview");
        assert_eq!(api_path(CLUSTER_CONFIG_GET_PATH), "/api/cluster/config/get");
        assert_eq!(api_path(CLUSTER_USER_LIST_PATH), "/api/cluster/user/list");
    }

    #[test]
    fn test_path_constants() {
        // Verify all paths start with /
        assert!(STATUS_PATH.starts_with('/'));
        assert!(MQTT_OVERVIEW_PATH.starts_with('/'));
        assert!(CLUSTER_CONFIG_SET_PATH.starts_with('/'));

        // Verify paths don't already contain /api prefix
        assert!(!MQTT_OVERVIEW_PATH.starts_with("/api"));
        assert!(!CLUSTER_CONFIG_GET_PATH.starts_with("/api"));
    }
}
