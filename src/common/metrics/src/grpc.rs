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

// ── Labels ──────────────────────────────────────────────────────────────────

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct GrpcMethodLabel {
    pub service: String,
    pub method: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct GrpcErrorLabel {
    pub service: String,
    pub method: String,
    pub status_code: String,
}

// ── Metrics (Server-side) ────────────────────────────────────────────────────

register_counter_metric!(
    GRPC_REQUESTS_TOTAL,
    "grpc_requests",
    "Total number of gRPC requests by service and method",
    GrpcMethodLabel
);

register_histogram_metric_ms_with_default_buckets!(
    GRPC_REQUEST_DURATION_MS,
    "grpc_request_duration_ms",
    "gRPC request duration in milliseconds by service and method",
    GrpcMethodLabel
);

register_counter_metric!(
    GRPC_ERRORS_TOTAL,
    "grpc_errors",
    "Total number of gRPC errors by service, method and status code",
    GrpcErrorLabel
);

// ── Metrics (Client-side) ───────────────────────────────────────────────────

register_histogram_metric_ms_with_default_buckets!(
    GRPC_CLIENT_CALL_DURATION_MS,
    "grpc_client_call_duration_ms",
    "gRPC client call duration in milliseconds by service and method",
    GrpcMethodLabel
);

// ── Public API ──────────────────────────────────────────────────────────────

pub fn record_grpc_client_call(service: &str, method: &str, duration_ms: f64) {
    let label = GrpcMethodLabel {
        service: service.to_string(),
        method: method.to_string(),
    };
    histogram_metric_observe!(GRPC_CLIENT_CALL_DURATION_MS, duration_ms, label);
}

pub fn record_grpc_request(service: &str, method: &str, status_code: &str, duration_ms: f64) {
    let method_label = GrpcMethodLabel {
        service: service.to_string(),
        method: method.to_string(),
    };

    gauge_metric_inc!(GRPC_REQUESTS_TOTAL, method_label);
    histogram_metric_observe!(GRPC_REQUEST_DURATION_MS, duration_ms, method_label);

    if status_code != "OK" {
        let error_label = GrpcErrorLabel {
            service: service.to_string(),
            method: method.to_string(),
            status_code: status_code.to_string(),
        };
        gauge_metric_inc!(GRPC_ERRORS_TOTAL, error_label);
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grpc_path() {
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

        assert!(parse_grpc_path("/").is_err());
        assert!(parse_grpc_path("/service").is_err());
        assert!(parse_grpc_path("").is_err());
        assert!(parse_grpc_path("/service/").is_err());
    }
}
