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
use crate::core::limit::is_subscribe_rate_exceeded;
use crate::core::pkid_manager::{PkidAckEnum, ReceiveQosPkidData};
use crate::core::sub_exclusive::{allow_exclusive_subscribe, already_exclusive_subscribe};
use crate::core::sub_wildcards::sub_path_validator;
use crate::core::subscribe::remove_subscribe;
use crate::core::subscribe::{save_subscribe, SaveSubscribeContext};
use crate::security::AuthDriver;
use crate::subscribe::common::min_qos;
use crate::subscribe::manager::SubscribeManager;
use crate::system_topic::event::{
    st_report_subscribed_event, st_report_unsubscribed_event, StReportSubscribedEventContext,
    StReportUnsubscribedEventContext,
};
use common_base::tools::now_second;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    qos, MqttPacket, MqttProtocol, SubAck, SubAckProperties, Subscribe, SubscribeProperties,
    SubscribeReasonCode, UnsubAck, UnsubAckProperties, UnsubAckReason, Unsubscribe,
    UnsubscribeProperties,
};
use std::sync::Arc;

impl MqttService {
    pub async fn subscribe(
        &self,
        connection: &MQTTConnection,
        subscribe: &Subscribe,
        subscribe_properties: &Option<SubscribeProperties>,
    ) -> MqttPacket {
        let (reason_codes, reason) = subscribe_validator(
            &self.cache_manager,
            &self.auth_driver,
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

        self.cache_manager.pkid_data.add_receive_publish_pkid_data(
            &connection.client_id,
            ReceiveQosPkidData {
                ack_enum: PkidAckEnum::SubAck,
                pkid: subscribe.packet_identifier,
                create_time: now_second(),
            },
        );

        if let Err(e) = save_subscribe(SaveSubscribeContext {
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

        if let Err(e) = self
            .retain_message_manager
            .try_send_retain_message(
                &connection.client_id,
                subscribe,
                subscribe_properties,
                &self.cache_manager,
                &self.subscribe_manager,
            )
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
        let cluster_qos = self
            .cache_manager
            .broker_cache
            .get_cluster_config()
            .await
            .mqtt_protocol_config
            .max_qos_flight_message;
        for filter in &subscribe.filters {
            match min_qos(qos(cluster_qos).unwrap(), filter.qos) {
                protocol::mqtt::common::QoS::AtMostOnce => {
                    return_codes.push(SubscribeReasonCode::QoS0);
                }
                protocol::mqtt::common::QoS::AtLeastOnce => {
                    return_codes.push(SubscribeReasonCode::QoS1);
                }
                protocol::mqtt::common::QoS::ExactlyOnce => {
                    return_codes.push(SubscribeReasonCode::QoS2);
                }
            }
        }

        self.cache_manager
            .pkid_data
            .remove_receive_publish_pkid_data(&connection.client_id, subscribe.packet_identifier);
        st_report_subscribed_event(StReportSubscribedEventContext {
            storage_driver_manager: self.storage_driver_manager.clone(),
            metadata_cache: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
            connection: connection.clone(),
            connect_id: connection.connect_id,
            connection_manager: self.connection_manager.clone(),
            subscribe: subscribe.clone(),
        })
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
        let (reason_codes, reason) =
            un_subscribe_validator(&connection.client_id, &self.subscribe_manager, un_subscribe);

        // Check if all validations passed
        let all_success = reason_codes.iter().all(|r| *r == UnsubAckReason::Success);

        if !all_success {
            // Validation failed for one or more filters
            return response_packet_mqtt_unsub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                un_subscribe.pkid,
                reason_codes,
                Some(reason),
            );
        }

        self.cache_manager.pkid_data.add_receive_publish_pkid_data(
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
            .pkid_data
            .remove_receive_publish_pkid_data(&connection.client_id, un_subscribe.pkid);

        st_report_unsubscribed_event(StReportUnsubscribedEventContext {
            storage_driver_manager: self.storage_driver_manager.clone(),
            metadata_cache: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
            connection: connection.clone(),
            connect_id: connection.connect_id,
            connection_manager: self.connection_manager.clone(),
            un_subscribe: un_subscribe.clone(),
        })
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
    auth_driver: &Arc<AuthDriver>,
    subscribe_manager: &Arc<SubscribeManager>,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
    subscribe_properties: &Option<SubscribeProperties>,
    protocol: &MqttProtocol,
) -> (Vec<SubscribeReasonCode>, String) {
    if subscribe.packet_identifier == 0 {
        return (
            vec![SubscribeReasonCode::PkidInUse],
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
                        vec![SubscribeReasonCode::SubscriptionIdNotSupported],
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
        .pkid_data
        .get_receive_publish_pkid_data(&connection.client_id, subscribe.packet_identifier)
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

    if is_subscribe_rate_exceeded() {
        return (
            vec![SubscribeReasonCode::QuotaExceeded],
            "Subscribe rate limit exceeded".to_string(),
        );
    }

    if !allow_exclusive_subscribe(subscribe) {
        return (
            vec![SubscribeReasonCode::ExclusiveSubscriptionDisabled],
            "Exclusive subscription is disabled".to_string(),
        );
    }

    if already_exclusive_subscribe(subscribe_manager, &connection.client_id, subscribe) {
        return (
            vec![SubscribeReasonCode::TopicSubscribed],
            "Topic already has an exclusive subscription".to_string(),
        );
    }

    if !auth_driver
        .auth_subscribe_check(connection, subscribe)
        .await
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
            vec![],
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
        if subscribe_manager.get_subscribe(client_id, path).is_none() {
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
