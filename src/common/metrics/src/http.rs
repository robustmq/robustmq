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
pub struct HttpMethodLabel {
    pub method: String,
    pub uri: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct HttpErrorLabel {
    pub method: String,
    pub uri: String,
    pub status_code: String,
}

// ── Metrics ─────────────────────────────────────────────────────────────────

register_counter_metric!(
    HTTP_REQUESTS_TOTAL,
    "http_requests",
    "Total number of HTTP requests by method and uri",
    HttpMethodLabel
);

register_histogram_metric_ms_with_default_buckets!(
    HTTP_REQUEST_DURATION_MS,
    "http_request_duration_ms",
    "HTTP request duration in milliseconds by method and uri",
    HttpMethodLabel
);

register_counter_metric!(
    HTTP_ERRORS_TOTAL,
    "http_errors",
    "Total number of HTTP errors by method, uri and status code",
    HttpErrorLabel
);

// ── Public API ──────────────────────────────────────────────────────────────

pub fn record_http_request(method: &str, uri: &str, status_code: u16, duration_ms: f64) {
    let method_label = HttpMethodLabel {
        method: method.to_string(),
        uri: uri.to_string(),
    };

    gauge_metric_inc!(HTTP_REQUESTS_TOTAL, method_label);
    histogram_metric_observe!(HTTP_REQUEST_DURATION_MS, duration_ms, method_label);

    if status_code >= 400 {
        let error_label = HttpErrorLabel {
            method: method.to_string(),
            uri: uri.to_string(),
            status_code: status_code.to_string(),
        };
        gauge_metric_inc!(HTTP_ERRORS_TOTAL, error_label);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_labels() {
        let method_label = HttpMethodLabel {
            uri: "/api/users".to_string(),
            method: "GET".to_string(),
        };
        assert_eq!(method_label.uri, "/api/users");
        assert_eq!(method_label.method, "GET");

        let error_label = HttpErrorLabel {
            uri: "/api/users".to_string(),
            method: "GET".to_string(),
            status_code: "500".to_string(),
        };
        assert_eq!(error_label.status_code, "500");
    }
}
