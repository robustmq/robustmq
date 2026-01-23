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

use crate::core::response::{
    response_packet_mqtt_distinct_by_reason, response_packet_mqtt_suback,
    response_packet_mqtt_unsuback,
};
use crate::core::retain::{is_new_sub, try_send_retain_message, TrySendRetainMessageContext};
use crate::core::subscribe::remove_subscribe;
use crate::core::subscribe::{save_subscribe, SaveSubscribeContext};
use crate::core::validator::{subscribe_validator, un_subscribe_validator};
use crate::subscribe::common::min_qos;
use crate::system_topic::event::{
    st_report_subscribed_event, st_report_unsubscribed_event, StReportSubscribedEventContext,
    StReportUnsubscribedEventContext,
};

use protocol::mqtt::common::{
    qos, DisconnectReasonCode, MqttPacket, Subscribe, SubscribeProperties, SubscribeReasonCode,
    UnsubAckReason, Unsubscribe, UnsubscribeProperties,
};

impl MqttService {
    pub async fn subscribe(
        &self,
        connect_id: u64,
        subscribe: &Subscribe,
        subscribe_properties: &Option<SubscribeProperties>,
    ) -> MqttPacket {
        let connection = if let Some(se) = self.cache_manager.get_connection(connect_id) {
            se.clone()
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
                None,
            );
        };

        if let Some(packet) = subscribe_validator(
            &self.protocol,
            &self.auth_driver,
            &self.client_pool,
            &self.subscribe_manager,
            &connection,
            subscribe,
        )
        .await
        {
            return packet;
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
            return response_packet_mqtt_suback(
                &self.protocol,
                &connection,
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
            connect_id,
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
        response_packet_mqtt_suback(
            &self.protocol,
            &connection,
            subscribe.packet_identifier,
            return_codes,
            None,
        )
    }

    pub async fn un_subscribe(
        &self,
        connect_id: u64,
        un_subscribe: &Unsubscribe,
        _: &Option<UnsubscribeProperties>,
    ) -> MqttPacket {
        let connection = if let Some(se) = self.cache_manager.get_connection(connect_id) {
            se.clone()
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
                None,
            );
        };

        if let Some(packet) = un_subscribe_validator(
            &connection.client_id,
            &self.subscribe_manager,
            &connection,
            un_subscribe,
        )
        .await
        {
            return packet;
        }

        if let Err(e) =
            remove_subscribe(&connection.client_id, un_subscribe, &self.client_pool).await
        {
            return response_packet_mqtt_unsuback(
                &connection,
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
            connect_id,
            connection_manager: self.connection_manager.clone(),
            un_subscribe: un_subscribe.clone(),
        })
        .await;

        response_packet_mqtt_unsuback(
            &connection,
            un_subscribe.pkid,
            vec![UnsubAckReason::Success],
            None,
        )
    }
}
