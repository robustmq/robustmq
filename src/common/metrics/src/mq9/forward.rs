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

//! Metrics for the Mq9 forward / fork-write pipeline.

use crate::{
    counter_metric_inc_by, histogram_metric_observe, register_counter_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct ForwardRuleLabel {
    pub tenant: String,
    pub rule_name: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct ForwardRuleFailureLabel {
    pub tenant: String,
    pub rule_name: String,
    pub reason: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct Mq9SendLabel {
    pub forked: String,
}

register_counter_metric!(
    MQ9_FORWARD_MATCH_TOTAL,
    "mq9_forward_match_total",
    "Total number of times a forward rule matched on the send path",
    ForwardRuleLabel
);

register_counter_metric!(
    MQ9_FORWARD_WRITE_SUCCESS_TOTAL,
    "mq9_forward_write_success_total",
    "Total number of fork-writes that completed successfully",
    ForwardRuleLabel
);

register_counter_metric!(
    MQ9_FORWARD_WRITE_FAILURE_TOTAL,
    "mq9_forward_write_failure_total",
    "Total number of fork-writes that failed, labeled by reason",
    ForwardRuleFailureLabel
);

register_histogram_metric_ms_with_default_buckets!(
    MQ9_FORWARD_WRITE_DURATION_MS,
    "mq9_forward_write_duration_ms",
    "Duration of a single fork-write attempt in milliseconds",
    ForwardRuleLabel
);

register_histogram_metric_ms_with_default_buckets!(
    MQ9_SEND_DURATION_MS,
    "mq9_send_duration_ms",
    "Duration of process_send in milliseconds, labelled by whether the send triggered a fork",
    Mq9SendLabel
);

pub fn record_forward_match(tenant: &str, rule_name: &str, count: u64) {
    let label = ForwardRuleLabel {
        tenant: tenant.to_string(),
        rule_name: rule_name.to_string(),
    };
    counter_metric_inc_by!(MQ9_FORWARD_MATCH_TOTAL, label, count);
}

pub fn record_forward_write_success(tenant: &str, rule_name: &str, duration_ms: f64) {
    let label = ForwardRuleLabel {
        tenant: tenant.to_string(),
        rule_name: rule_name.to_string(),
    };
    counter_metric_inc_by!(MQ9_FORWARD_WRITE_SUCCESS_TOTAL, label, 1);
    histogram_metric_observe!(MQ9_FORWARD_WRITE_DURATION_MS, duration_ms, label);
}

pub fn record_forward_write_failure(
    tenant: &str,
    rule_name: &str,
    reason: &str,
    duration_ms: f64,
) {
    let label = ForwardRuleFailureLabel {
        tenant: tenant.to_string(),
        rule_name: rule_name.to_string(),
        reason: reason.to_string(),
    };
    counter_metric_inc_by!(MQ9_FORWARD_WRITE_FAILURE_TOTAL, label, 1);

    let dur_label = ForwardRuleLabel {
        tenant: tenant.to_string(),
        rule_name: rule_name.to_string(),
    };
    histogram_metric_observe!(MQ9_FORWARD_WRITE_DURATION_MS, duration_ms, dur_label);
}

pub fn record_send_duration(forked: bool, duration_ms: f64) {
    let label = Mq9SendLabel {
        forked: if forked { "true" } else { "false" }.to_string(),
    };
    histogram_metric_observe!(MQ9_SEND_DURATION_MS, duration_ms, label);
}
