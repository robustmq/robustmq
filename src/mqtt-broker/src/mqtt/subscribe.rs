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
use crate::core::flow_control::is_subscribe_rate_exceeded;
use crate::core::retain::{is_new_sub, try_send_retain_message, TrySendRetainMessageContext};
use crate::core::sub_exclusive::{allow_exclusive_subscribe, already_exclusive_subscribe};
use crate::core::sub_share::group_leader_validator;
use crate::core::sub_wildcards::sub_path_validator;
use crate::core::subscribe::remove_subscribe;
use crate::core::subscribe::{save_subscribe, SaveSubscribeContext};
use crate::mqtt::disconnect::build_distinct_packet;
use crate::security::AuthDriver;
use crate::subscribe::common::min_qos;
use crate::subscribe::manager::SubscribeManager;
use crate::system_topic::event::{
    st_report_subscribed_event, st_report_unsubscribed_event, StReportSubscribedEventContext,
    StReportUnsubscribedEventContext,
};
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    qos, DisconnectReasonCode, MqttPacket, MqttProtocol, SubAck, SubAckProperties, Subscribe,
    SubscribeProperties, SubscribeReasonCode, UnsubAck, UnsubAckProperties, UnsubAckReason,
    Unsubscribe, UnsubscribeProperties,
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
            &self.auth_driver,
            &self.subscribe_manager,
            connection,
            subscribe,
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

        match group_leader_validator(&self.client_pool, &subscribe.filters).await {
            Ok(Some(addr)) => {
                return build_distinct_packet(
                    &self.cache_manager,
                    connection.connect_id,
                    &self.protocol,
                    Some(DisconnectReasonCode::UseAnotherServer),
                    Some(addr),
                    Some(
                        "group leader assigned; please reconnect to the provided server"
                            .to_string(),
                    ),
                );
            }
            Ok(None) => {}
            Err(e) => {
                return response_packet_mqtt_sub_ack(
                    &self.cache_manager,
                    connection.connect_id,
                    &self.protocol,
                    subscribe.packet_identifier,
                    vec![SubscribeReasonCode::Unspecified],
                    Some(e.to_string()),
                );
            }
        }

        let new_subs = is_new_sub(&connection.client_id, subscribe, &self.subscribe_manager).await;

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

        try_send_retain_message(TrySendRetainMessageContext {
            protocol: self.protocol.clone(),
            client_id: connection.client_id.clone(),
            subscribe: subscribe.clone(),
            subscribe_properties: subscribe_properties.clone(),
            client_pool: self.client_pool.clone(),
            cache_manager: self.cache_manager.clone(),
            connection_manager: self.connection_manager.clone(),
            is_new_subs: new_subs,
        })
        .await;

        let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
        let cluster_qos = self
            .cache_manager
            .broker_cache
            .get_cluster_config()
            .await
            .mqtt_protocol_config
            .max_qos;
        for filter in subscribe.filters.clone() {
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
            un_subscribe_validator(&connection.client_id, &self.subscribe_manager, un_subscribe)
                .await;

        if !reason_codes.is_empty() {
            return response_packet_mqtt_unsub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                un_subscribe.pkid,
                reason_codes,
                Some(reason),
            );
        }

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
    auth_driver: &Arc<AuthDriver>,
    subscribe_manager: &Arc<SubscribeManager>,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
) -> (Vec<SubscribeReasonCode>, String) {
    let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
    let mut invalid_paths = Vec::new();

    for filter in subscribe.filters.clone() {
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

async fn un_subscribe_validator(
    client_id: &str,
    subscribe_manager: &Arc<SubscribeManager>,
    un_subscribe: &Unsubscribe,
) -> (Vec<UnsubAckReason>, String) {
    let mut return_codes: Vec<UnsubAckReason> = Vec::new();
    let mut invalid_paths = Vec::new();
    for path in un_subscribe.filters.clone() {
        if sub_path_validator(&path).is_err() {
            return_codes.push(UnsubAckReason::TopicFilterInvalid);
            invalid_paths.push(path.clone());
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

    for path in un_subscribe.filters.clone() {
        if subscribe_manager.get_subscribe(client_id, &path).is_none() {
            return (
                vec![UnsubAckReason::NoSubscriptionExisted],
                MqttBrokerError::SubscriptionPathNotExists(path).to_string(),
            );
        }
    }

    (Vec::new(), "".to_string())
}
