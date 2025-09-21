use prometheus_client::encoding::EncodeLabelSet;

use crate::{gauge_metric_inc, register_counter_metric};

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct HttpLabel {
    pub uri: String,
}

register_counter_metric!(
    HTTP_REQUEST_NUM,
    "http_request_num",
    "Number of calls to the http request",
    HttpLabel
);

register_counter_metric!(
    HTTP_REQUEST_SUCCESS_NUM,
    "http_request_success_num",
    "Number of calls to the success http request",
    HttpLabel
);

register_counter_metric!(
    HTTP_REQUEST_ERROR_NUM,
    "http_request_error_num",
    "Number of calls to the success http request",
    HttpLabel
);

pub fn metrics_http_request_incr(uri: String) {
    let label = HttpLabel {
        uri,
        ..Default::default()
    };
    gauge_metric_inc!(HTTP_REQUEST_NUM, label)
}

pub fn metrics_http_request_success_incr(uri: String) {
    let label = HttpLabel {
        uri,
        ..Default::default()
    };
    gauge_metric_inc!(HTTP_REQUEST_SUCCESS_NUM, label)
}

pub fn metrics_http_request_error_incr(uri: String) {
    let label = HttpLabel {
        uri,
        ..Default::default()
    };
    gauge_metric_inc!(HTTP_REQUEST_ERROR_NUM, label)
}
