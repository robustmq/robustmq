use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use crate::handler::constant::{METRICS_KEY_LABLE_NAME, METRICS_KEY_TYPE_NAME};

lazy_static! {
    static ref BROKER_NETWORK_QUEUE_NUM: IntGaugeVec = register_int_gauge_vec!(
        "network_queue_num",
        "broker network queue num",
        &[METRICS_KEY_LABLE_NAME, METRICS_KEY_TYPE_NAME]
    )
    .unwrap();
}

pub fn metrics_request_queue(lable: &str, len: usize) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&[lable, "request"])
        .set(len as i64);
}

pub fn metrics_response_queue(lable: &str, len: usize) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&[lable, "response"])
        .set(len as i64);
}
