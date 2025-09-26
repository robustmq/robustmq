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

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct HttpBasicLabel {
    pub uri: String,
    pub method: String,
}

register_counter_metric!(
    HTTP_REQUEST_TOTAL,
    "http_requests_total",
    "Total number of HTTP requests by method, uri and status code",
    HttpLabel
);

register_histogram_metric_ms_with_default_buckets!(
    HTTP_REQUEST_DURATION_MS,
    "http_request_duration_milliseconds",
    "HTTP request duration in milliseconds",
    HttpBasicLabel
);

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

register_counter_metric!(
    HTTP_REQUEST_ERRORS_TOTAL,
    "http_request_errors_total",
    "Total number of HTTP request errors by type",
    HttpLabel
);

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
}
