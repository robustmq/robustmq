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
    gauge_metric_inc, histogram_metric_observe, register_counter_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct GrpcLabel {
    /// gRPC service name (e.g., "placement.center.kv.KvService")
    pub service: String,
    /// gRPC method name (e.g., "get", "set")
    pub method: String,
    /// gRPC status code (e.g., "OK", "INVALID_ARGUMENT", "INTERNAL")
    pub status_code: String,
}

register_counter_metric!(
    GRPC_REQUESTS_TOTAL,
    "grpc_requests_total",
    "Total number of gRPC requests by service, method and status code",
    GrpcLabel
);

register_histogram_metric_ms_with_default_buckets!(
    GRPC_REQUEST_DURATION_MS,
    "grpc_request_duration_milliseconds",
    "gRPC request duration in milliseconds",
    GrpcLabel
);

register_histogram_metric_ms_with_default_buckets!(
    GRPC_REQUEST_SIZE_BYTES,
    "grpc_request_size_bytes",
    "gRPC request size in bytes",
    GrpcLabel
);

register_histogram_metric_ms_with_default_buckets!(
    GRPC_RESPONSE_SIZE_BYTES,
    "grpc_response_size_bytes",
    "gRPC response size in bytes",
    GrpcLabel
);

register_counter_metric!(
    GRPC_REQUEST_ERRORS_TOTAL,
    "grpc_request_errors_total",
    "Total number of gRPC request errors by type",
    GrpcLabel
);

pub fn record_grpc_request(
    service: String,
    method: String,
    status_code: String,
    duration_ms: f64,
    request_size_bytes: Option<f64>,
    response_size_bytes: Option<f64>,
) {
    // Record total requests
    let label = GrpcLabel {
        service: service.clone(),
        method: method.clone(),
        status_code: status_code.clone(),
    };
    gauge_metric_inc!(GRPC_REQUESTS_TOTAL, label);
    histogram_metric_observe!(GRPC_REQUEST_DURATION_MS, duration_ms, label);

    // Record request/response sizes if provided
    if let Some(req_size) = request_size_bytes {
        histogram_metric_observe!(GRPC_REQUEST_SIZE_BYTES, req_size, label);
    }
    if let Some(resp_size) = response_size_bytes {
        histogram_metric_observe!(GRPC_RESPONSE_SIZE_BYTES, resp_size, label);
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

pub fn record_grpc_error(service: String, method: String, status_code: String) {
    let label = GrpcLabel {
        service,
        method,
        status_code,
    };
    gauge_metric_inc!(GRPC_REQUEST_ERRORS_TOTAL, label);
}

pub fn parse_grpc_path(uri: &str) -> Result<(String, String), &'static str> {
    let path = uri.trim_start_matches('/');
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err("Invalid gRPC path format")
    }
}

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
}
