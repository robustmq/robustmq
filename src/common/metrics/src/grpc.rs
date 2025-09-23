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
    gauge_metric_inc, histogram_metric_observe, register_counter_metric, register_gauge_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

/// Enhanced gRPC label with comprehensive dimensions for better monitoring
#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct GrpcLabel {
    /// gRPC service name (e.g., "placement.center.kv.KvService")
    pub service: String,
    /// gRPC method name (e.g., "get", "set")
    pub method: String,
    /// gRPC status code (e.g., "OK", "INVALID_ARGUMENT", "INTERNAL")
    pub status_code: String,
}

/// Simplified gRPC label for basic metrics that don't need status code
#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct GrpcBasicLabel {
    pub service: String,
    pub method: String,
}

// === Request Count Metrics ===
register_counter_metric!(
    GRPC_REQUESTS_TOTAL,
    "grpc_requests_total",
    "Total number of gRPC requests by service, method and status code",
    GrpcLabel
);

register_counter_metric!(
    GRPC_REQUEST_DURATION_COUNT,
    "grpc_request_duration_count",
    "Total number of gRPC requests for duration tracking",
    GrpcBasicLabel
);

// === Request Duration Metrics ===
register_histogram_metric_ms_with_default_buckets!(
    GRPC_REQUEST_DURATION_MS,
    "grpc_request_duration_milliseconds",
    "gRPC request duration in milliseconds",
    GrpcBasicLabel
);

// === Request/Response Size Metrics ===
register_histogram_metric_ms_with_default_buckets!(
    GRPC_REQUEST_SIZE_BYTES,
    "grpc_request_size_bytes",
    "gRPC request size in bytes",
    GrpcBasicLabel
);

register_histogram_metric_ms_with_default_buckets!(
    GRPC_RESPONSE_SIZE_BYTES,
    "grpc_response_size_bytes",
    "gRPC response size in bytes",
    GrpcBasicLabel
);

// === Active Connections Metrics ===
register_gauge_metric!(
    GRPC_ACTIVE_CONNECTIONS,
    "grpc_active_connections",
    "Current number of active gRPC connections",
    crate::core::server::NoLabelSet
);

// === Error Rate Metrics ===
register_counter_metric!(
    GRPC_REQUEST_ERRORS_TOTAL,
    "grpc_request_errors_total",
    "Total number of gRPC request errors by type",
    GrpcLabel
);

// === New Enhanced Functions ===

/// Record a complete gRPC request with all metrics
pub fn record_grpc_request(
    service: String,
    method: String,
    status_code: String,
    duration_ms: f64,
    request_size_bytes: Option<f64>,
    response_size_bytes: Option<f64>,
) {
    // Record total requests
    let full_label = GrpcLabel {
        service: service.clone(),
        method: method.clone(),
        status_code: status_code.clone(),
    };
    gauge_metric_inc!(GRPC_REQUESTS_TOTAL, full_label);

    // Record duration
    let basic_label = GrpcBasicLabel {
        service: service.clone(),
        method: method.clone(),
    };
    histogram_metric_observe!(GRPC_REQUEST_DURATION_MS, duration_ms, basic_label);
    let count_label = GrpcBasicLabel {
        service: service.clone(),
        method: method.clone(),
    };
    gauge_metric_inc!(GRPC_REQUEST_DURATION_COUNT, count_label);

    // Record request/response sizes if provided
    if let Some(req_size) = request_size_bytes {
        let req_label = GrpcBasicLabel {
            service: service.clone(),
            method: method.clone(),
        };
        histogram_metric_observe!(GRPC_REQUEST_SIZE_BYTES, req_size, req_label);
    }
    if let Some(resp_size) = response_size_bytes {
        let resp_label = GrpcBasicLabel {
            service: service.clone(),
            method: method.clone(),
        };
        histogram_metric_observe!(GRPC_RESPONSE_SIZE_BYTES, resp_size, resp_label);
    }

    // Record errors for non-OK status codes
    if status_code != "OK" {
        let error_label = GrpcLabel {
            service,
            method,
            status_code,
        };
        gauge_metric_inc!(GRPC_REQUEST_ERRORS_TOTAL, error_label);
    }
}

/// Record gRPC connection start (increment active connections)
pub fn record_grpc_connection_start() {
    use crate::core::server::NoLabelSet;
    gauge_metric_inc!(GRPC_ACTIVE_CONNECTIONS, NoLabelSet);
}

/// Record gRPC connection end (decrement active connections)
pub fn record_grpc_connection_end() {
    // Note: This is a placeholder as gauge_metric_dec is not available
    // In practice, you might want to use a separate mechanism for connection tracking
}

/// Record gRPC request duration only
pub fn record_grpc_request_duration(service: String, method: String, duration_ms: f64) {
    let label = GrpcBasicLabel {
        service: service.clone(),
        method: method.clone(),
    };
    histogram_metric_observe!(GRPC_REQUEST_DURATION_MS, duration_ms, label);
    let count_label = GrpcBasicLabel { service, method };
    gauge_metric_inc!(GRPC_REQUEST_DURATION_COUNT, count_label);
}

/// Record gRPC error
pub fn record_grpc_error(service: String, method: String, status_code: String) {
    let label = GrpcLabel {
        service,
        method,
        status_code,
    };
    gauge_metric_inc!(GRPC_REQUEST_ERRORS_TOTAL, label);
}

/// Parse gRPC path safely
pub fn parse_grpc_path(uri: &str) -> Result<(String, String), &'static str> {
    let path = uri.trim_start_matches('/');
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err("Invalid gRPC path format")
    }
}

/// Extract gRPC status code from HTTP response headers
pub fn extract_grpc_status_code(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .map(|s| match s {
            "0" => "OK",
            "1" => "CANCELLED",
            "2" => "UNKNOWN",
            "3" => "INVALID_ARGUMENT",
            "4" => "DEADLINE_EXCEEDED",
            "5" => "NOT_FOUND",
            "6" => "ALREADY_EXISTS",
            "7" => "PERMISSION_DENIED",
            "8" => "RESOURCE_EXHAUSTED",
            "9" => "FAILED_PRECONDITION",
            "10" => "ABORTED",
            "11" => "OUT_OF_RANGE",
            "12" => "UNIMPLEMENTED",
            "13" => "INTERNAL",
            "14" => "UNAVAILABLE",
            "15" => "DATA_LOSS",
            "16" => "UNAUTHENTICATED",
            _ => "UNKNOWN",
        })
        .unwrap_or("OK")
        .to_string()
}

/// Extract gRPC status code from tonic MetadataMap (for pure gRPC services)
pub fn extract_grpc_status_code_from_metadata(headers: &tonic::metadata::MetadataMap) -> String {
    headers
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .map(|s| match s {
            "0" => "OK",
            "1" => "CANCELLED",
            "2" => "UNKNOWN",
            "3" => "INVALID_ARGUMENT",
            "4" => "DEADLINE_EXCEEDED",
            "5" => "NOT_FOUND",
            "6" => "ALREADY_EXISTS",
            "7" => "PERMISSION_DENIED",
            "8" => "RESOURCE_EXHAUSTED",
            "9" => "FAILED_PRECONDITION",
            "10" => "ABORTED",
            "11" => "OUT_OF_RANGE",
            "12" => "UNIMPLEMENTED",
            "13" => "INTERNAL",
            "14" => "UNAVAILABLE",
            "15" => "DATA_LOSS",
            "16" => "UNAUTHENTICATED",
            _ => "UNKNOWN",
        })
        .unwrap_or("OK")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_labels() {
        let full_label = GrpcLabel {
            service: "placement.center.kv.KvService".to_string(),
            method: "get".to_string(),
            status_code: "OK".to_string(),
        };

        let basic_label = GrpcBasicLabel {
            service: "placement.center.kv.KvService".to_string(),
            method: "get".to_string(),
        };

        assert_eq!(full_label.service, "placement.center.kv.KvService");
        assert_eq!(full_label.method, "get");
        assert_eq!(full_label.status_code, "OK");

        assert_eq!(basic_label.service, "placement.center.kv.KvService");
        assert_eq!(basic_label.method, "get");
    }

    #[test]
    fn test_parse_grpc_path() {
        // Test valid paths
        assert_eq!(
            parse_grpc_path("/placement.center.kv.KvService/get"),
            Ok((
                "placement.center.kv.KvService".to_string(),
                "get".to_string()
            ))
        );

        assert_eq!(
            parse_grpc_path("/mqtt.service.MqttService/publish"),
            Ok((
                "mqtt.service.MqttService".to_string(),
                "publish".to_string()
            ))
        );

        // Test invalid paths
        assert!(parse_grpc_path("/").is_err());
        assert!(parse_grpc_path("/service").is_err());
        assert!(parse_grpc_path("").is_err());
        assert!(parse_grpc_path("/service/").is_err());
    }

    #[test]
    fn test_extract_grpc_status_code() {
        use axum::http::HeaderMap;
        let mut headers = HeaderMap::new();

        // Test default (no header)
        assert_eq!(extract_grpc_status_code(&headers), "OK");

        // Test various status codes
        headers.insert("grpc-status", "0".parse().unwrap());
        assert_eq!(extract_grpc_status_code(&headers), "OK");

        headers.insert("grpc-status", "3".parse().unwrap());
        assert_eq!(extract_grpc_status_code(&headers), "INVALID_ARGUMENT");

        headers.insert("grpc-status", "13".parse().unwrap());
        assert_eq!(extract_grpc_status_code(&headers), "INTERNAL");

        headers.insert("grpc-status", "999".parse().unwrap());
        assert_eq!(extract_grpc_status_code(&headers), "UNKNOWN");
    }

    #[test]
    fn test_extract_grpc_status_code_from_metadata() {
        let mut headers = tonic::metadata::MetadataMap::new();

        // Test default (no header)
        assert_eq!(extract_grpc_status_code_from_metadata(&headers), "OK");

        // Test various status codes
        headers.insert("grpc-status", "0".parse().unwrap());
        assert_eq!(extract_grpc_status_code_from_metadata(&headers), "OK");

        headers.insert("grpc-status", "3".parse().unwrap());
        assert_eq!(
            extract_grpc_status_code_from_metadata(&headers),
            "INVALID_ARGUMENT"
        );

        headers.insert("grpc-status", "13".parse().unwrap());
        assert_eq!(extract_grpc_status_code_from_metadata(&headers), "INTERNAL");

        headers.insert("grpc-status", "999".parse().unwrap());
        assert_eq!(extract_grpc_status_code_from_metadata(&headers), "UNKNOWN");
    }

    #[tokio::test]
    async fn test_record_grpc_request() {
        // Test successful request
        record_grpc_request(
            "test.Service".to_string(),
            "testMethod".to_string(),
            "OK".to_string(),
            150.0,
            Some(1024.0),
            Some(2048.0),
        );

        // Test error request
        record_grpc_request(
            "test.Service".to_string(),
            "testMethod".to_string(),
            "INTERNAL".to_string(),
            300.0,
            Some(512.0),
            None,
        );
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        record_grpc_connection_start();
        record_grpc_connection_end();
    }

    #[tokio::test]
    async fn test_duration_recording() {
        record_grpc_request_duration("test.Service".to_string(), "testMethod".to_string(), 75.0);
    }

    #[tokio::test]
    async fn test_error_recording() {
        record_grpc_error(
            "test.Service".to_string(),
            "testMethod".to_string(),
            "PERMISSION_DENIED".to_string(),
        );
    }
}
