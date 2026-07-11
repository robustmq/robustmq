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

use super::MqttService;
use crate::core::cache::MQTTCacheManager;
use crate::core::connection::is_request_problem_info;
use crate::core::error::MqttBrokerError;
use crate::core::event::{st_report_subscribed_event, st_report_unsubscribed_event};
use crate::core::pkid_manager::{PkidAckEnum, ReceiveQosPkidData};
use crate::core::security::security_is_allow_subscribe;
use crate::core::sub_exclusive::{allow_exclusive_subscribe, already_exclusive_subscribe};
use crate::core::sub_share::{
    decode_share_info, full_group_name, is_mqtt_share_subscribe, resolve_share_sub_leader_id,
};
use crate::core::sub_wildcards::sub_path_validator;
use crate::core::subscribe::remove_subscribe;
use crate::core::subscribe::{save_subscribe, SaveSubscribeContext};
use crate::subscribe::common::min_qos;
use crate::subscribe::manager::SubscribeManager;
use broker_core::share_group::ShareGroupStorage;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use common_security::manager::SecurityManager;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::share_group::ShareGroupParams;
use protocol::mqtt::common::{
    Disconnect, DisconnectProperties, DisconnectReasonCode, MqttPacket, MqttProtocol, QoS, SubAck,
    SubAckProperties, Subscribe, SubscribeProperties, SubscribeReasonCode, UnsubAck,
    UnsubAckProperties, UnsubAckReason, Unsubscribe, UnsubscribeProperties,
};
use std::sync::Arc;
use tracing::{info, warn};

impl MqttService {
    pub async fn subscribe(
        &self,
        connection: &MQTTConnection,
        subscribe: &Subscribe,
        subscribe_properties: &Option<SubscribeProperties>,
    ) -> MqttPacket {
        let (reason_codes, reason) = subscribe_validator(
            &self.cache_manager,
            &self.security_manager,
            &self.subscribe_manager,
            connection,
            subscribe,
            subscribe_properties,
            &self.protocol,
        )
        .await;

        if !reason_codes.is_empty() {
            return response_packet_mqtt_sub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                subscribe.packet_identifier,
                reason_codes,
                Some(reason),
            );
        }

        // MQTT5 share-subscription leader redirect: a share group is served by exactly
        // one broker (its leader_broker). Ensure the group exists (which assigns the
        // leader), then if this node is not the leader, redirect the client to the leader
        // via a ServerMoved DISCONNECT so it reconnects to the broker that pushes the group.
        self.ensure_share_groups_exist(&connection.tenant, subscribe)
            .await;
        if self.protocol.is_mqtt5() {
            if let Some(server_ref) = self
                .share_sub_redirect_target(&connection.tenant, subscribe)
                .await
            {
                info!(
                    "Redirecting client '{}' share subscription to leader broker at {}",
                    connection.client_id, server_ref
                );
                return MqttPacket::Disconnect(
                    Disconnect {
                        reason_code: Some(DisconnectReasonCode::ServerMoved),
                    },
                    Some(DisconnectProperties {
                        server_reference: Some(server_ref),
                        ..Default::default()
                    }),
                );
            }
        }

        self.cache_manager.pkid_manager.add_qos_pkid_data(
            &connection.client_id,
            ReceiveQosPkidData {
                ack_enum: PkidAckEnum::SubAck,
                pkid: subscribe.packet_identifier,
                create_time: now_second(),
            },
        );

        if let Err(e) = save_subscribe(SaveSubscribeContext {
            tenant: connection.tenant.clone(),
            client_id: connection.client_id.clone(),
            protocol: self.protocol.clone(),
            client_pool: self.client_pool.clone(),
            cache_manager: self.cache_manager.clone(),
            subscribe_manager: self.subscribe_manager.clone(),
            subscribe: subscribe.clone(),
            subscribe_properties: subscribe_properties.clone(),
        })
        .await
        {
            return response_packet_mqtt_sub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                subscribe.packet_identifier,
                vec![SubscribeReasonCode::Unspecified],
                Some(e.to_string()),
            );
        }

        if let Err(e) =
            crate::core::retain::try_send_retain_message(crate::core::retain::SendRetainContext {
                storage_driver_manager: &self.storage_driver_manager,
                cache_manager: &self.cache_manager,
                connection_manager: &self.connection_manager,
                subscribe_manager: &self.subscribe_manager,
                tenant: &connection.tenant,
                client_id: &connection.client_id,
                subscribe,
                stop_sx: &self.stop_sx,
            })
            .await
        {
            return response_packet_mqtt_sub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                subscribe.packet_identifier,
                vec![SubscribeReasonCode::Unspecified],
                Some(e.to_string()),
            );
        }

        let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
        for filter in &subscribe.filters {
            match min_qos(QoS::ExactlyOnce, filter.qos) {
                QoS::AtMostOnce => {
                    return_codes.push(SubscribeReasonCode::QoS0);
                }
                QoS::AtLeastOnce => {
                    return_codes.push(SubscribeReasonCode::QoS1);
                }
                QoS::ExactlyOnce => {
                    return_codes.push(SubscribeReasonCode::QoS2);
                }
            }
        }

        self.cache_manager
            .pkid_manager
            .remove_qos_pkid_data(&connection.client_id, subscribe.packet_identifier);
        st_report_subscribed_event(
            &self.event_manager,
            &self.connection_manager,
            connection.connect_id,
            connection,
            subscribe,
        )
        .await;

        response_packet_mqtt_sub_ack(
            &self.cache_manager,
            connection.connect_id,
            &self.protocol,
            subscribe.packet_identifier,
            return_codes,
            None,
        )
    }

    pub async fn un_subscribe(
        &self,
        connection: &MQTTConnection,
        un_subscribe: &Unsubscribe,
        _: &Option<UnsubscribeProperties>,
    ) -> MqttPacket {
        let (reason_codes, reason) = un_subscribe_validator(
            &connection.tenant,
            &connection.client_id,
            &self.subscribe_manager,
            un_subscribe,
        );

        let all_success = reason_codes.iter().all(|r| *r == UnsubAckReason::Success);
        if !all_success {
            return response_packet_mqtt_unsub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                un_subscribe.pkid,
                reason_codes,
                Some(reason),
            );
        }

        self.cache_manager.pkid_manager.add_qos_pkid_data(
            &connection.client_id,
            ReceiveQosPkidData {
                ack_enum: PkidAckEnum::SubAck,
                pkid: un_subscribe.pkid,
                create_time: now_second(),
            },
        );

        if let Err(e) =
            remove_subscribe(&connection.client_id, un_subscribe, &self.client_pool).await
        {
            return response_packet_mqtt_unsub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                un_subscribe.pkid,
                vec![UnsubAckReason::UnspecifiedError],
                Some(e.to_string()),
            );
        }

        self.cache_manager
            .pkid_manager
            .remove_qos_pkid_data(&connection.client_id, un_subscribe.pkid);

        st_report_unsubscribed_event(
            &self.event_manager,
            &self.connection_manager,
            connection.connect_id,
            connection,
            un_subscribe,
        )
        .await;

        response_packet_mqtt_unsub_ack(
            &self.cache_manager,
            connection.connect_id,
            &self.protocol,
            un_subscribe.pkid,
            vec![UnsubAckReason::Success],
            None,
        )
    }

    /// If any `$share` filter in this subscribe belongs to a group whose leader is a
    /// different broker, return that leader's MQTT address (for an MQTT5 ServerMoved
    /// redirect). Returns `None` when this node leads every share group in the request.
    async fn share_sub_redirect_target(
        &self,
        tenant: &str,
        subscribe: &Subscribe,
    ) -> Option<String> {
        let local_broker_id = broker_config().broker_id;
        for filter in &subscribe.filters {
            if !is_mqtt_share_subscribe(&filter.path) {
                continue;
            }
            let (group_name, sub_name) = decode_share_info(&filter.path);
            let group_name_full = full_group_name(&group_name, &sub_name);

            let leader_id = match resolve_share_sub_leader_id(
                &self.cache_manager,
                &self.client_pool,
                tenant,
                &group_name_full,
            )
            .await
            {
                Ok(Some(id)) => id,
                _ => continue,
            };
            if leader_id == local_broker_id {
                continue;
            }
            if let Some(node) = self.cache_manager.node_cache.node_lists.get(&leader_id) {
                let addr = node.extend.mqtt.mqtt_addr.clone();
                if !addr.is_empty() {
                    return Some(addr);
                }
            }
        }
        None
    }

    async fn ensure_share_groups_exist(&self, tenant: &str, subscribe: &Subscribe) {
        for filter in &subscribe.filters {
            if !is_mqtt_share_subscribe(&filter.path) {
                continue;
            }
            let (group_name, sub_name) = decode_share_info(&filter.path);
            let group_name_full = full_group_name(&group_name, &sub_name);
            if self
                .cache_manager
                .node_cache
                .get_share_group(tenant, &group_name_full)
                .is_some()
            {
                continue;
            }
            let storage = ShareGroupStorage::new(self.client_pool.clone());
            if let Err(e) = storage
                .create(
                    tenant,
                    &group_name_full,
                    ShareGroupParams::MQTT(
                        metadata_struct::mqtt::share_group::ShareGroupParamsMqtt {},
                    ),
                )
                .await
            {
                warn!(
                    "Failed to create share group '{}' for tenant '{}': {}",
                    group_name_full, tenant, e
                );
            }
        }
    }
}

fn response_packet_mqtt_sub_ack(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
    pkid: u16,
    return_codes: Vec<SubscribeReasonCode>,
    reason_string: Option<String>,
) -> MqttPacket {
    let sub_ack = SubAck { pkid, return_codes };
    if !protocol.is_mqtt5() {
        return MqttPacket::SubAck(sub_ack, None);
    }

    let mut properties = SubAckProperties::default();
    if is_request_problem_info(cache_manager, connect_id) {
        properties.reason_string = reason_string;
    }

    MqttPacket::SubAck(sub_ack, Some(properties))
}

fn response_packet_mqtt_unsub_ack(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
    pkid: u16,
    reasons: Vec<UnsubAckReason>,
    reason_string: Option<String>,
) -> MqttPacket {
    let unsub_ack = UnsubAck { pkid, reasons };
    if !protocol.is_mqtt5() {
        return MqttPacket::UnsubAck(unsub_ack, None);
    }

    let mut properties = UnsubAckProperties::default();
    if is_request_problem_info(cache_manager, connect_id) {
        properties.reason_string = reason_string;
    }
    MqttPacket::UnsubAck(unsub_ack, None)
}

async fn subscribe_validator(
    cache_manager: &Arc<MQTTCacheManager>,
    security_manager: &Arc<SecurityManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
    subscribe_properties: &Option<SubscribeProperties>,
    protocol: &MqttProtocol,
) -> (Vec<SubscribeReasonCode>, String) {
    if subscribe.packet_identifier == 0 {
        return (
            vec![SubscribeReasonCode::Unspecified],
            "Packet identifier must be non-zero".to_string(),
        );
    }

    if subscribe.filters.is_empty() {
        return (
            vec![SubscribeReasonCode::TopicFilterInvalid],
            "Subscription must contain at least one topic filter".to_string(),
        );
    }

    if let Some(properties) = subscribe_properties {
        if let Some(sub_id) = properties.subscription_identifier {
            if protocol.is_mqtt5() {
                if sub_id == 0 || sub_id > 268_435_455 {
                    return (
                        vec![SubscribeReasonCode::TopicFilterInvalid],
                        format!(
                            "Subscription identifier must be in range 1-268435455, got {}",
                            sub_id
                        ),
                    );
                }
            } else if sub_id != 0 {
                return (
                    vec![SubscribeReasonCode::SubscriptionIdNotSupported],
                    "Subscription identifier not supported in MQTT 3.1.1/4".to_string(),
                );
            }
        }
    }

    if cache_manager
        .pkid_manager
        .get_qos_pkid_data(&connection.client_id, subscribe.packet_identifier)
        .is_some()
    {
        return (
            vec![SubscribeReasonCode::PkidInUse],
            "Packet identifier already in use".to_string(),
        );
    }

    let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
    let mut invalid_paths = Vec::new();

    for filter in &subscribe.filters {
        if sub_path_validator(&filter.path).is_err() {
            return_codes.push(SubscribeReasonCode::TopicFilterInvalid);
            invalid_paths.push(filter.path.clone());
            continue;
        }
    }

    if !return_codes.is_empty() {
        let error_msg = if invalid_paths.len() == 1 {
            MqttBrokerError::InvalidSubPath(invalid_paths[0].clone()).to_string()
        } else {
            format!("Invalid topic filter(s): {}", invalid_paths.join(", "))
        };
        return (return_codes, error_msg);
    }

    if !allow_exclusive_subscribe(subscribe) {
        return (
            vec![SubscribeReasonCode::ExclusiveSubscriptionDisabled],
            "Exclusive subscription is disabled".to_string(),
        );
    }

    if already_exclusive_subscribe(
        subscribe_manager,
        &connection.tenant,
        &connection.client_id,
        subscribe,
    ) {
        return (
            vec![SubscribeReasonCode::TopicSubscribed],
            "Topic already has an exclusive subscription".to_string(),
        );
    }

    if !security_is_allow_subscribe(cache_manager, security_manager, connection, subscribe)
        .await
        .unwrap_or(false)
    {
        return (
            vec![SubscribeReasonCode::NotAuthorized],
            "Subscription not authorized".to_string(),
        );
    }

    (Vec::new(), "".to_string())
}

/// Validates an UNSUBSCRIBE packet according to MQTT protocol requirements.
///
/// This function checks:
/// 1. Packet identifier must be non-zero
/// 2. Must contain at least one topic filter
/// 3. Each topic filter must have valid format
/// 4. Each topic filter must correspond to an existing subscription
///
/// According to MQTT 5.0 specification, the validator returns a reason code
/// for each topic filter in the UNSUBSCRIBE packet, allowing partial success.
///
/// # Arguments
/// * `client_id` - The client identifier
/// * `subscribe_manager` - Manager containing active subscriptions
/// * `un_subscribe` - The UNSUBSCRIBE packet to validate
///
/// # Returns
/// A tuple of (Vec<UnsubAckReason>, String):
/// - Vec<UnsubAckReason>: One reason code per topic filter
/// - String: Error message if any validation failed (empty on success)
fn un_subscribe_validator(
    tenant: &str,
    client_id: &str,
    subscribe_manager: &Arc<SubscribeManager>,
    un_subscribe: &Unsubscribe,
) -> (Vec<UnsubAckReason>, String) {
    // Validate packet identifier (MQTT protocol requirement)
    if un_subscribe.pkid == 0 {
        // Return error reason code for all filters
        return (
            vec![UnsubAckReason::UnspecifiedError; un_subscribe.filters.len()],
            "Packet identifier must be non-zero".to_string(),
        );
    }

    // Validate that at least one topic filter is present
    if un_subscribe.filters.is_empty() {
        return (
            vec![UnsubAckReason::TopicFilterInvalid],
            "UNSUBSCRIBE must contain at least one topic filter".to_string(),
        );
    }

    let mut return_codes: Vec<UnsubAckReason> = Vec::with_capacity(un_subscribe.filters.len());
    let mut has_error = false;
    let mut error_details: Vec<String> = Vec::new();

    // Validate each topic filter individually
    for path in &un_subscribe.filters {
        // Check topic filter format validity
        if sub_path_validator(path).is_err() {
            return_codes.push(UnsubAckReason::TopicFilterInvalid);
            error_details.push(format!("Invalid topic filter: {}", path));
            has_error = true;
            continue;
        }

        // Check if subscription exists
        if subscribe_manager
            .get_subscribe(tenant, client_id, path)
            .is_none()
        {
            return_codes.push(UnsubAckReason::NoSubscriptionExisted);
            error_details.push(format!("Subscription not found: {}", path));
            has_error = true;
            continue;
        }

        // Validation passed for this filter
        return_codes.push(UnsubAckReason::Success);
    }

    let error_msg = if has_error {
        error_details.join("; ")
    } else {
        String::new()
    };

    (return_codes, error_msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tool::test_build_mqtt_cache_manager;
    use common_base::tools::now_second;
    use common_security::manager::SecurityManager;
    use dashmap::DashMap;
    use metadata_struct::mqtt::connection::MQTTConnection;
    use protocol::mqtt::common::{
        Filter, MqttProtocol, QoS, Subscribe, SubscribeProperties, Unsubscribe,
    };
    use std::sync::Arc;

    fn build_test_connection() -> MQTTConnection {
        MQTTConnection {
            connect_id: 1,
            tenant: "test_tenant".to_string(),
            client_id: "test_client".to_string(),
            is_login: true,
            source_ip_addr: "127.0.0.1".to_string(),
            source_ip: "127.0.0.1".to_string(),
            clean_session: true,
            login_user: None,
            keep_alive: 60,
            topic_alias: DashMap::new(),
            client_max_receive_maximum: 100,
            max_packet_size: 1024 * 1024,
            topic_alias_max: 10,
            request_problem_info: 1,
            create_time: now_second(),
        }
    }

    fn build_test_subscribe(pkid: u16, filters: Vec<Filter>) -> Subscribe {
        Subscribe {
            packet_identifier: pkid,
            filters,
        }
    }

    fn build_test_filter(path: &str, qos: QoS) -> Filter {
        Filter {
            path: path.to_string(),
            qos,
            ..Default::default()
        }
    }

    async fn run_subscribe_validator(
        subscribe: Subscribe,
        properties: Option<SubscribeProperties>,
        protocol: MqttProtocol,
    ) -> (Vec<SubscribeReasonCode>, String) {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let security_manager = Arc::new(SecurityManager::new());
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let connection = build_test_connection();

        subscribe_validator(
            &cache_manager,
            &security_manager,
            &subscribe_manager,
            &connection,
            &subscribe,
            &properties,
            &protocol,
        )
        .await
    }

    fn sub_id_props(id: usize) -> SubscribeProperties {
        SubscribeProperties {
            subscription_identifier: Some(id),
            ..Default::default()
        }
    }

    // =========================================================================
    // SUBSCRIBE validator tests
    // =========================================================================

    /// pkid=0 is a malformed packet, not "PkidInUse".
    /// Expected: `Unspecified` (0x80), not `PkidInUse` (0x91).
    #[tokio::test]
    async fn subscribe_validator_pkid_zero_returns_unspecified() {
        let subscribe = build_test_subscribe(0, vec![build_test_filter("/test", QoS::AtLeastOnce)]);
        let (codes, msg) = run_subscribe_validator(subscribe, None, MqttProtocol::Mqtt5).await;

        assert_eq!(codes, vec![SubscribeReasonCode::Unspecified]);
        assert!(msg.contains("non-zero"));
    }

    /// Subscription identifier 0 is out of the valid range (1–268435455).
    /// Expected: `TopicFilterInvalid` (0x8F), not `SubscriptionIdNotSupported` (0xA1).
    #[tokio::test]
    async fn subscribe_validator_sub_id_zero_returns_topic_filter_invalid() {
        let subscribe = build_test_subscribe(1, vec![build_test_filter("/test", QoS::AtLeastOnce)]);
        let (codes, _) =
            run_subscribe_validator(subscribe, Some(sub_id_props(0)), MqttProtocol::Mqtt5).await;

        assert_eq!(codes, vec![SubscribeReasonCode::TopicFilterInvalid]);
    }

    /// Subscription identifier above max (268435456) is out of range.
    /// Expected: `TopicFilterInvalid` (0x8F).
    #[tokio::test]
    async fn subscribe_validator_sub_id_above_max_returns_topic_filter_invalid() {
        let subscribe = build_test_subscribe(1, vec![build_test_filter("/test", QoS::AtLeastOnce)]);
        let (codes, _) = run_subscribe_validator(
            subscribe,
            Some(sub_id_props(268_435_456)),
            MqttProtocol::Mqtt5,
        )
        .await;

        assert_eq!(codes, vec![SubscribeReasonCode::TopicFilterInvalid]);
    }

    /// Empty filter list is rejected with `TopicFilterInvalid`.
    #[tokio::test]
    async fn subscribe_validator_empty_filters_returns_topic_filter_invalid() {
        let subscribe = build_test_subscribe(1, vec![]);
        let (codes, msg) = run_subscribe_validator(subscribe, None, MqttProtocol::Mqtt5).await;

        assert_eq!(codes, vec![SubscribeReasonCode::TopicFilterInvalid]);
        assert!(!msg.is_empty());
    }

    /// Malformed topic filter (e.g. `#` not at end) is rejected.
    #[tokio::test]
    async fn subscribe_validator_invalid_topic_filter_path() {
        let subscribe =
            build_test_subscribe(1, vec![build_test_filter("/test/#/bad", QoS::AtLeastOnce)]);
        let (codes, _) = run_subscribe_validator(subscribe, None, MqttProtocol::Mqtt5).await;

        assert_eq!(codes, vec![SubscribeReasonCode::TopicFilterInvalid]);
    }

    /// Normal subscribe with a valid filter succeeds (returns empty codes vec).
    #[tokio::test]
    async fn subscribe_validator_success_returns_empty() {
        let subscribe =
            build_test_subscribe(1, vec![build_test_filter("/test/topic", QoS::AtLeastOnce)]);
        let (codes, msg) = run_subscribe_validator(subscribe, None, MqttProtocol::Mqtt5).await;

        assert!(codes.is_empty(), "expected empty vec, got {:?}", codes);
        assert!(msg.is_empty());
    }

    /// A valid subscription identifier (within range) succeeds.
    #[tokio::test]
    async fn subscribe_validator_valid_sub_id_succeeds() {
        let subscribe =
            build_test_subscribe(1, vec![build_test_filter("/test/topic", QoS::AtLeastOnce)]);
        let (codes, msg) =
            run_subscribe_validator(subscribe, Some(sub_id_props(42)), MqttProtocol::Mqtt5).await;

        assert!(codes.is_empty(), "expected empty vec, got {:?}", codes);
        assert!(msg.is_empty());
    }

    /// MQTT v3.1.1/4 rejects subscription identifiers entirely.
    #[tokio::test]
    async fn subscribe_validator_nonzero_sub_id_rejected_in_mqtt4() {
        let subscribe =
            build_test_subscribe(1, vec![build_test_filter("/test/topic", QoS::AtLeastOnce)]);
        let (codes, _) =
            run_subscribe_validator(subscribe, Some(sub_id_props(1)), MqttProtocol::Mqtt4).await;

        assert_eq!(codes, vec![SubscribeReasonCode::SubscriptionIdNotSupported]);
    }

    // =========================================================================
    // UNSUBSCRIBE validator tests
    // =========================================================================
    #[test]
    fn un_subscribe_validator_empty_filters_returns_error() {
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let un_subscribe = Unsubscribe {
            pkid: 1,
            filters: vec![],
        };

        let (codes, msg) =
            un_subscribe_validator("tenant", "client", &subscribe_manager, &un_subscribe);

        assert!(
            !codes.is_empty(),
            "BUG: empty filters must return at least one error reason code"
        );
        assert_eq!(codes[0], UnsubAckReason::TopicFilterInvalid);
        assert!(msg.contains("at least one"));
    }

    /// pkid=0 is a malformed UNSUBSCRIBE packet.
    #[test]
    fn un_subscribe_validator_pkid_zero_returns_unspecified_error() {
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let un_subscribe = Unsubscribe {
            pkid: 0,
            filters: vec!["/test".to_string()],
        };

        let (codes, msg) =
            un_subscribe_validator("tenant", "client", &subscribe_manager, &un_subscribe);

        assert_eq!(codes, vec![UnsubAckReason::UnspecifiedError]);
        assert!(msg.contains("non-zero"));
    }

    /// Invalid topic filter format is rejected per-filter.
    #[test]
    fn un_subscribe_validator_invalid_topic_filter() {
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let un_subscribe = Unsubscribe {
            pkid: 1,
            filters: vec!["/test/#/bad".to_string()],
        };

        let (codes, _) =
            un_subscribe_validator("tenant", "client", &subscribe_manager, &un_subscribe);

        assert_eq!(codes, vec![UnsubAckReason::TopicFilterInvalid]);
    }

    /// Topic filter without an active subscription returns NoSubscriptionExisted.
    #[test]
    fn un_subscribe_validator_no_subscription_existed() {
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let un_subscribe = Unsubscribe {
            pkid: 1,
            filters: vec!["/nonexistent".to_string()],
        };

        let (codes, _) =
            un_subscribe_validator("tenant", "client", &subscribe_manager, &un_subscribe);

        assert_eq!(codes, vec![UnsubAckReason::NoSubscriptionExisted]);
    }

    #[test]
    fn un_subscribe_validator_mixed_valid_and_invalid() {
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let un_subscribe = Unsubscribe {
            pkid: 1,
            filters: vec!["/valid/topic".to_string(), "/bad/#/invalid".to_string()],
        };

        let (codes, _) =
            un_subscribe_validator("tenant", "client", &subscribe_manager, &un_subscribe);

        // /valid/topic has no subscription -> NoSubscriptionExisted
        // /bad/#/invalid is malformed -> TopicFilterInvalid
        assert_eq!(
            codes,
            vec![
                UnsubAckReason::NoSubscriptionExisted,
                UnsubAckReason::TopicFilterInvalid,
            ]
        );
    }
}
