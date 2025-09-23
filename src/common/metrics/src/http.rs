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

use prometheus_client::encoding::EncodeLabelSet;

use crate::{
    gauge_metric_inc, histogram_metric_observe, register_counter_metric, register_gauge_metric,
    register_histogram_metric_ms_with_default_buckets,
};

/// Enhanced HTTP label with comprehensive dimensions for better monitoring
#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct HttpLabel {
    /// Request URI/endpoint
    pub uri: String,
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    /// HTTP status code (200, 404, 500, etc.)
    pub status_code: String,
}

/// Simplified HTTP label for basic metrics that don't need status code
#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct HttpBasicLabel {
    pub uri: String,
    pub method: String,
}

// === Request Count Metrics ===
register_counter_metric!(
    HTTP_REQUEST_TOTAL,
    "http_requests_total",
    "Total number of HTTP requests by method, uri and status code",
    HttpLabel
);

register_counter_metric!(
    HTTP_REQUEST_DURATION_COUNT,
    "http_request_duration_count",
    "Total number of HTTP requests for duration tracking",
    HttpBasicLabel
);

// === Request Duration Metrics ===
register_histogram_metric_ms_with_default_buckets!(
    HTTP_REQUEST_DURATION_MS,
    "http_request_duration_milliseconds",
    "HTTP request duration in milliseconds",
    HttpBasicLabel
);

// === Request/Response Size Metrics ===
register_histogram_metric_ms_with_default_buckets!(
    HTTP_REQUEST_SIZE_BYTES,
    "http_request_size_bytes",
    "HTTP request size in bytes",
    HttpBasicLabel
);

register_histogram_metric_ms_with_default_buckets!(
    HTTP_RESPONSE_SIZE_BYTES,
    "http_response_size_bytes",
    "HTTP response size in bytes",
    HttpBasicLabel
);

// === Active Connections Metrics ===
register_gauge_metric!(
    HTTP_ACTIVE_CONNECTIONS,
    "http_active_connections",
    "Current number of active HTTP connections",
    crate::core::server::NoLabelSet
);

// === Error Rate Metrics ===
register_counter_metric!(
    HTTP_REQUEST_ERRORS_TOTAL,
    "http_request_errors_total",
    "Total number of HTTP request errors by type",
    HttpLabel
);

// === Helper Functions ===

/// Record a complete HTTP request with all metrics
pub fn record_http_request(
    method: String,
    uri: String,
    status_code: u16,
    duration_ms: f64,
    request_size_bytes: Option<f64>,
    response_size_bytes: Option<f64>,
) {
    let status_str = status_code.to_string();

    // Record total requests
    let full_label = HttpLabel {
        method: method.clone(),
        uri: uri.clone(),
        status_code: status_str.clone(),
    };
    gauge_metric_inc!(HTTP_REQUEST_TOTAL, full_label);

    // Record duration
    let basic_label = HttpBasicLabel {
        method: method.clone(),
        uri: uri.clone(),
    };
    histogram_metric_observe!(HTTP_REQUEST_DURATION_MS, duration_ms, basic_label);
    gauge_metric_inc!(HTTP_REQUEST_DURATION_COUNT, basic_label);

    // Record request/response sizes if provided
    if let Some(req_size) = request_size_bytes {
        let req_label = basic_label.clone();
        histogram_metric_observe!(HTTP_REQUEST_SIZE_BYTES, req_size, req_label);
    }
    if let Some(resp_size) = response_size_bytes {
        let resp_label = basic_label.clone();
        histogram_metric_observe!(HTTP_RESPONSE_SIZE_BYTES, resp_size, resp_label);
    }

    // Record errors for non-2xx status codes
    if status_code >= 400 {
        let error_label = HttpLabel {
            method,
            uri,
            status_code: status_str,
        };
        gauge_metric_inc!(HTTP_REQUEST_ERRORS_TOTAL, error_label);
    }
}

/// Record HTTP request start (increment active connections)
pub fn record_http_connection_start() {
    use crate::core::server::NoLabelSet;
    gauge_metric_inc!(HTTP_ACTIVE_CONNECTIONS, NoLabelSet);
}

/// Record HTTP request end (decrement active connections)  
pub fn record_http_connection_end() {
    // Note: gauge_metric_dec is not available, using inc with -1 would require different approach
    // For now, this function is a placeholder for connection tracking
    // In practice, you might want to use a separate gauge that tracks active connections
}

/// Record HTTP request duration only
pub fn record_http_request_duration(method: String, uri: String, duration_ms: f64) {
    let label = HttpBasicLabel {
        method: method.clone(),
        uri: uri.clone(),
    };
    histogram_metric_observe!(HTTP_REQUEST_DURATION_MS, duration_ms, label);
    let count_label = HttpBasicLabel { method, uri };
    gauge_metric_inc!(HTTP_REQUEST_DURATION_COUNT, count_label);
}

/// Record HTTP error
pub fn record_http_error(method: String, uri: String, status_code: u16) {
    let label = HttpLabel {
        method,
        uri,
        status_code: status_code.to_string(),
    };
    gauge_metric_inc!(HTTP_REQUEST_ERRORS_TOTAL, label);
}

// === Legacy Functions (Deprecated) ===
// These functions are kept for backward compatibility but should be replaced
// with the new record_http_* functions

#[deprecated(note = "Use record_http_request instead")]
pub fn metrics_http_request_incr(uri: String) {
    let label = HttpLabel {
        uri,
        method: "UNKNOWN".to_string(),
        status_code: "200".to_string(),
    };
    gauge_metric_inc!(HTTP_REQUEST_TOTAL, label);
}

#[deprecated(note = "Use record_http_request instead")]
pub fn metrics_http_request_success_incr(uri: String) {
    let label = HttpLabel {
        uri,
        method: "UNKNOWN".to_string(),
        status_code: "200".to_string(),
    };
    gauge_metric_inc!(HTTP_REQUEST_TOTAL, label);
}

#[deprecated(note = "Use record_http_error instead")]
pub fn metrics_http_request_error_incr(uri: String) {
    let label = HttpLabel {
        uri,
        method: "UNKNOWN".to_string(),
        status_code: "500".to_string(),
    };
    gauge_metric_inc!(HTTP_REQUEST_ERRORS_TOTAL, label);
}

#[deprecated(note = "Use record_http_request_duration instead")]
pub fn metrics_http_request_ms(uri: String, ms: f64) {
    let label = HttpBasicLabel {
        uri,
        method: "UNKNOWN".to_string(),
    };
    histogram_metric_observe!(HTTP_REQUEST_DURATION_MS, ms, label);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_labels() {
        let full_label = HttpLabel {
            uri: "/api/users".to_string(),
            method: "GET".to_string(),
            status_code: "200".to_string(),
        };

        let basic_label = HttpBasicLabel {
            uri: "/api/users".to_string(),
            method: "GET".to_string(),
        };

        assert_eq!(full_label.uri, "/api/users");
        assert_eq!(full_label.method, "GET");
        assert_eq!(full_label.status_code, "200");

        assert_eq!(basic_label.uri, "/api/users");
        assert_eq!(basic_label.method, "GET");
    }

    #[tokio::test]
    async fn test_record_http_request() {
        // Test successful request
        record_http_request(
            "GET".to_string(),
            "/api/users".to_string(),
            200,
            150.0,
            Some(1024.0),
            Some(2048.0),
        );

        // Test error request
        record_http_request(
            "POST".to_string(),
            "/api/users".to_string(),
            500,
            300.0,
            Some(512.0),
            None,
        );
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        record_http_connection_start();
        record_http_connection_end();
    }

    #[tokio::test]
    async fn test_duration_recording() {
        record_http_request_duration("PUT".to_string(), "/api/users/123".to_string(), 75.0);
    }

    #[tokio::test]
    async fn test_error_recording() {
        record_http_error("DELETE".to_string(), "/api/users/456".to_string(), 404);
    }
}
