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

// Cluster API paths
pub const CLUSTER_CONFIG_SET_PATH: &str = "/cluster/config/set";
pub const CLUSTER_CONFIG_GET_PATH: &str = "/cluster/config/get";

// MQTT Overview API paths
pub const MQTT_OVERVIEW_PATH: &str = "/mqtt/overview";
pub const MQTT_MONITOR_PATH: &str = "/mqtt/monitor/data";

// MQTT Client API paths
pub const MQTT_CLIENT_LIST_PATH: &str = "/mqtt/client/list";

// MQTT Session API paths
pub const MQTT_SESSION_LIST_PATH: &str = "/mqtt/session/list";

// MQTT Topic API paths
pub const MQTT_TOPIC_LIST_PATH: &str = "/mqtt/topic/list";
pub const MQTT_TOPIC_DETAIL_PATH: &str = "/mqtt/topic/detail";
pub const MQTT_TOPIC_DELETE_PATH: &str = "/mqtt/topic/delete";

// MQTT Topic Rewrite API paths
pub const MQTT_TOPIC_REWRITE_LIST_PATH: &str = "/mqtt/topic-rewrite/list";
pub const MQTT_TOPIC_REWRITE_CREATE_PATH: &str = "/mqtt/topic-rewrite/create";
pub const MQTT_TOPIC_REWRITE_DELETE_PATH: &str = "/mqtt/topic-rewrite/delete";

// MQTT Subscribe API paths
pub const MQTT_SUBSCRIBE_LIST_PATH: &str = "/mqtt/subscribe/list";
pub const MQTT_SUBSCRIBE_DETAIL_PATH: &str = "/mqtt/subscribe/detail";
pub const MQTT_SHARE_SUBSCRIBE_DETAIL_PATH: &str = "/mqtt/subscribe/share-detail";

// MQTT Auto Subscribe API paths
pub const MQTT_AUTO_SUBSCRIBE_LIST_PATH: &str = "/mqtt/auto-subscribe/list";
pub const MQTT_AUTO_SUBSCRIBE_CREATE_PATH: &str = "/mqtt/auto-subscribe/create";
pub const MQTT_AUTO_SUBSCRIBE_DELETE_PATH: &str = "/mqtt/auto-subscribe/delete";

// MQTT Slow Subscribe API paths
pub const MQTT_SLOW_SUBSCRIBE_LIST_PATH: &str = "/mqtt/slow-subscribe/list";

// MQTT User API paths
pub const MQTT_USER_LIST_PATH: &str = "/mqtt/user/list";
pub const MQTT_USER_CREATE_PATH: &str = "/mqtt/user/create";
pub const MQTT_USER_DELETE_PATH: &str = "/mqtt/user/delete";

// MQTT ACL API paths
pub const MQTT_ACL_LIST_PATH: &str = "/mqtt/acl/list";
pub const MQTT_ACL_CREATE_PATH: &str = "/mqtt/acl/create";
pub const MQTT_ACL_DELETE_PATH: &str = "/mqtt/acl/delete";

// MQTT Blacklist API paths
pub const MQTT_BLACKLIST_LIST_PATH: &str = "/mqtt/blacklist/list";
pub const MQTT_BLACKLIST_CREATE_PATH: &str = "/mqtt/blacklist/create";
pub const MQTT_BLACKLIST_DELETE_PATH: &str = "/mqtt/blacklist/delete";

// MQTT Flapping Detect API paths
pub const MQTT_FLAPPING_DETECT_LIST_PATH: &str = "/mqtt/flapping_detect/list";

// MQTT Connector API paths
pub const MQTT_CONNECTOR_LIST_PATH: &str = "/mqtt/connector/list";
pub const MQTT_CONNECTOR_CREATE_PATH: &str = "/mqtt/connector/create";
pub const MQTT_CONNECTOR_DETAIL_PATH: &str = "/mqtt/connector/detail";
pub const MQTT_CONNECTOR_DELETE_PATH: &str = "/mqtt/connector/delete";

// MQTT Schema API paths
pub const MQTT_SCHEMA_LIST_PATH: &str = "/mqtt/schema/list";
pub const MQTT_SCHEMA_CREATE_PATH: &str = "/mqtt/schema/create";
pub const MQTT_SCHEMA_DELETE_PATH: &str = "/mqtt/schema/delete";
pub const MQTT_SCHEMA_BIND_LIST_PATH: &str = "/mqtt/schema-bind/list";
pub const MQTT_SCHEMA_BIND_CREATE_PATH: &str = "/mqtt/schema-bind/create";
pub const MQTT_SCHEMA_BIND_DELETE_PATH: &str = "/mqtt/schema-bind/delete";

// MQTT System API paths
pub const MQTT_SYSTEM_ALARM_LIST_PATH: &str = "/mqtt/system-alarm/list";
pub const MQTT_BAN_LOG_LIST_PATH: &str = "/mqtt/ban-log/list";

// MQTT Message API paths
pub const MQTT_MESSAGE_SEND_PATH: &str = "/mqtt/message/send";
pub const MQTT_MESSAGE_READ_PATH: &str = "/mqtt/message/read";

// Utility functions for building API paths with prefix
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
        assert_eq!(api_path(MQTT_USER_LIST_PATH), "/api/mqtt/user/list");
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
