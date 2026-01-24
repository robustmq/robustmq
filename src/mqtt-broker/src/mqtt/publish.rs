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
use crate::core::delay_message::{decode_delay_topic, is_delay_topic};
use crate::core::metrics::record_publish_receive_metrics;
use crate::core::offline_message::{save_message, SaveMessageContext};
use crate::core::response::{
    build_pub_ack_fail, build_puback, build_pubrec, response_packet_mqtt_distinct_by_reason,
};
use crate::core::topic::{get_topic_name, try_init_topic};
use crate::core::validator::publish_validator;
use common_metrics::mqtt::publish::record_mqtt_messages_delayed_inc;
use protocol::mqtt::common::{
    DisconnectReasonCode, MqttPacket, PubAckReason, PubRecReason, Publish, PublishProperties, QoS,
};

impl MqttService {
    pub async fn publish(
        &self,
        connect_id: u64,
        publish: &Publish,
        publish_properties: &Option<PublishProperties>,
    ) -> Option<MqttPacket> {
        let connection = if let Some(se) = self.cache_manager.get_connection(connect_id) {
            se.clone()
        } else {
            return Some(response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
                None,
            ));
        };

        if let Some(pkg) = publish_validator(
            &self.protocol,
            &self.cache_manager,
            &connection,
            publish,
            publish_properties,
        )
        .await
        {
            if publish.qos == QoS::AtMostOnce {
                return None;
            } else {
                return Some(pkg);
            }
        }

        let is_pub_ack = publish.qos != QoS::ExactlyOnce;

        let mut topic_name = match get_topic_name(
            &self.cache_manager,
            connect_id,
            publish,
            publish_properties,
        )
        .await
        {
            Ok(topic_name) => topic_name,
            Err(e) => {
                return Some(build_pub_ack_fail(
                    &self.protocol,
                    &connection,
                    publish.p_kid,
                    Some(e.to_string()),
                    is_pub_ack,
                ))
            }
        };

        let mut delay_info = if is_delay_topic(&topic_name) {
            match decode_delay_topic(&topic_name) {
                Ok(data) => {
                    record_mqtt_messages_delayed_inc();
                    topic_name = data.target_topic_name.clone();
                    Some(data)
                }
                Err(e) => {
                    return Some(build_pub_ack_fail(
                        &self.protocol,
                        &connection,
                        publish.p_kid,
                        Some(e.to_string()),
                        is_pub_ack,
                    ))
                }
            }
        } else {
            None
        };

        if !self
            .auth_driver
            .auth_publish_check(&connection, &topic_name, publish.retain, publish.qos)
            .await
        {
            if is_pub_ack {
                return Some(build_puback(
                    &self.protocol,
                    &connection,
                    publish.p_kid,
                    PubAckReason::NotAuthorized,
                    None,
                    Vec::new(),
                ));
            } else {
                return Some(build_pubrec(
                    &self.protocol,
                    &connection,
                    publish.p_kid,
                    PubRecReason::NotAuthorized,
                    None,
                    Vec::new(),
                ));
            }
        }

        let topic = match try_init_topic(
            &topic_name,
            &self.cache_manager,
            &self.storage_driver_manager,
            &self.client_pool,
        )
        .await
        {
            Ok(tp) => tp,
            Err(e) => {
                return Some(build_pub_ack_fail(
                    &self.protocol,
                    &connection,
                    publish.p_kid,
                    Some(e.to_string()),
                    is_pub_ack,
                ))
            }
        };

        if delay_info.is_some() {
            let mut new_delay_info = delay_info.unwrap();
            new_delay_info.target_shard_name = Some(topic.topic_name.clone());
            delay_info = Some(new_delay_info);
        }

        if self.schema_manager.is_check_schema(&topic_name) {
            if let Err(e) = self.schema_manager.validate(&topic_name, &publish.payload) {
                return Some(build_pub_ack_fail(
                    &self.protocol,
                    &connection,
                    publish.p_kid,
                    Some(e.to_string()),
                    is_pub_ack,
                ));
            }
        }

        let client_id = connection.client_id.clone();

        // Persisting stores message data
        let offset = match save_message(SaveMessageContext {
            storage_driver_manager: self.storage_driver_manager.clone(),
            delay_message_manager: self.delay_message_manager.clone(),
            cache_manager: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
            publish: publish.clone(),
            publish_properties: publish_properties.clone(),
            subscribe_manager: self.subscribe_manager.clone(),
            client_id: client_id.clone(),
            topic: topic.clone(),
            delay_info,
        })
        .await
        {
            Ok(da) => {
                format!("{da:?}")
            }
            Err(e) => {
                return Some(build_pub_ack_fail(
                    &self.protocol,
                    &connection,
                    publish.p_kid,
                    Some(e.to_string()),
                    is_pub_ack,
                ))
            }
        };

        let user_properties: Vec<(String, String)> = vec![("offset".to_string(), offset)];

        self.cache_manager
            .add_topic_alias(connect_id, &topic_name, publish_properties);

        record_publish_receive_metrics(
            &client_id,
            connect_id,
            &topic_name,
            publish.payload.len() as u64,
        );

        match publish.qos {
            QoS::AtMostOnce => None,
            QoS::AtLeastOnce => Some(build_puback(
                &self.protocol,
                &connection,
                publish.p_kid,
                PubAckReason::Success,
                None,
                user_properties,
            )),
            QoS::ExactlyOnce => {
                self.cache_manager
                    .pkid_metadata
                    .add_client_pkid(&client_id, publish.p_kid);

                Some(build_pubrec(
                    &self.protocol,
                    &connection,
                    publish.p_kid,
                    PubRecReason::Success,
                    None,
                    user_properties,
                ))
            }
        }
    }
}
