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

//! Kafka client telemetry (KIP-714). RobustMQ answers the subscription
//! handshake honestly with "no metrics wanted" (`requested_metrics: []`,
//! a value the spec defines as "no metrics subscribed") rather than
//! rejecting the request outright — the protocol works end to end, clients
//! just have nothing to push. `PushTelemetryReq` and `ListConfigResourcesReq`
//! are consequently near-no-ops today. To later actually collect client
//! metrics: change `requested_metrics` below to `[""]` (subscribe to
//! everything) or a specific prefix list, then decode `PushTelemetryRequest
//! .metrics` (OpenTelemetry `MetricsData` protobuf — this repo has no
//! decoder for it yet, unlike the trace-only `opentelemetry` dependency
//! already in Cargo.toml) and forward/store the result somewhere.

use kafka_protocol::messages::{
    GetTelemetrySubscriptionsRequest, GetTelemetrySubscriptionsResponse,
    ListConfigResourcesRequest, ListConfigResourcesResponse, PushTelemetryRequest,
    PushTelemetryResponse,
};
use protocol::kafka::packet::KafkaPacket;
use uuid::Uuid;

// Kafka's `CompressionType` wire value for "no compression" — the only one
// worth accepting while we never decode a push's payload anyway.
const COMPRESSION_NONE: i8 = 0;

// Real Kafka clients don't push anything when they see an empty subscribed-metrics
// set, so this is mostly cosmetic; picked to match Kafka's own default.
const DEFAULT_PUSH_INTERVAL_MS: i32 = 300_000;
const DEFAULT_TELEMETRY_MAX_BYTES: i32 = 1024 * 1024;

pub fn process_get_telemetry_subscriptions(
    req: &GetTelemetrySubscriptionsRequest,
) -> Option<KafkaPacket> {
    // Per the response schema: the assigned id is only returned when the
    // client didn't already have one (request id == 0, the nil UUID); a
    // client that already has an instance id gets back the nil UUID here,
    // not its own id echoed back.
    let client_instance_id = if req.client_instance_id.is_nil() {
        Uuid::new_v4()
    } else {
        Uuid::nil()
    };

    Some(KafkaPacket::GetTelemetrySubscriptionsResponse(
        GetTelemetrySubscriptionsResponse::default()
            .with_error_code(0)
            .with_client_instance_id(client_instance_id)
            .with_subscription_id(0)
            .with_accepted_compression_types(vec![COMPRESSION_NONE])
            .with_push_interval_ms(DEFAULT_PUSH_INTERVAL_MS)
            .with_telemetry_max_bytes(DEFAULT_TELEMETRY_MAX_BYTES)
            .with_delta_temporality(false)
            .with_requested_metrics(vec![]),
    ))
}

// We never subscribe to any metrics (see module doc), so a spec-compliant
// client won't call this at all — this only exists to answer gracefully if
// one does anyway (e.g. a final push on disconnect). The payload is never
// decoded.
pub fn process_push_telemetry(_req: &PushTelemetryRequest) -> Option<KafkaPacket> {
    Some(KafkaPacket::PushTelemetryResponse(
        PushTelemetryResponse::default().with_error_code(0),
    ))
}

pub fn process_list_config_resources(_req: &ListConfigResourcesRequest) -> Option<KafkaPacket> {
    Some(KafkaPacket::ListConfigResourcesResponse(
        ListConfigResourcesResponse::default()
            .with_error_code(0)
            .with_config_resources(vec![]),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unwrap_get_subscriptions(packet: Option<KafkaPacket>) -> GetTelemetrySubscriptionsResponse {
        match packet {
            Some(KafkaPacket::GetTelemetrySubscriptionsResponse(resp)) => resp,
            other => panic!("expected GetTelemetrySubscriptionsResponse, got {other:?}"),
        }
    }

    #[test]
    fn new_client_gets_a_freshly_assigned_instance_id() {
        let req = GetTelemetrySubscriptionsRequest::default().with_client_instance_id(Uuid::nil());
        let resp = unwrap_get_subscriptions(process_get_telemetry_subscriptions(&req));
        assert!(!resp.client_instance_id.is_nil());
        assert!(resp.requested_metrics.is_empty());
    }

    #[test]
    fn returning_client_gets_nil_back_not_its_own_id() {
        let existing_id = Uuid::new_v4();
        let req = GetTelemetrySubscriptionsRequest::default().with_client_instance_id(existing_id);
        let resp = unwrap_get_subscriptions(process_get_telemetry_subscriptions(&req));
        assert!(resp.client_instance_id.is_nil());
        assert_ne!(resp.client_instance_id, existing_id);
    }
}
