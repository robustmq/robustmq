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
use crate::core::content_type::payload_format_indicator_check_by_publish;
use crate::core::delay_message::{decode_delay_topic, is_delay_topic};
use crate::core::error::MqttBrokerError;
use crate::core::flow_control::is_qos_message;
use crate::core::metrics::record_publish_receive_metrics;
use crate::core::offline_message::{save_message, SaveMessageContext};
use crate::core::topic::{get_topic_name, try_init_topic};
use common_metrics::mqtt::publish::record_mqtt_messages_delayed_inc;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    MqttPacket, MqttProtocol, PubAck, PubAckProperties, PubAckReason, PubRec, PubRecProperties,
    PubRecReason, Publish, PublishProperties, QoS,
};
use std::cmp::min;
use std::sync::Arc;
use tracing::debug;

impl MqttService {
    pub async fn publish(
        &self,
        connection: &MQTTConnection,
        publish: &Publish,
        publish_properties: &Option<PublishProperties>,
    ) -> Option<MqttPacket> {
        let is_pub_ack = publish.qos != QoS::ExactlyOnce;
        if let Some(reason_info) =
            publish_validator(&self.cache_manager, connection, publish, publish_properties).await
        {
            if publish.qos == QoS::AtMostOnce {
                return None;
            } else {
                return Some(build_pub_ack_fail(
                    &self.cache_manager,
                    connection.connect_id,
                    &self.protocol,
                    publish.p_kid,
                    reason_info,
                    is_pub_ack,
                ));
            }
        }

        let (offset, topic_name) = match self
            .process_publish0(connection, publish, publish_properties)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                let (pub_rec_reason, pub_ack_reason) = match &e {
                    MqttBrokerError::NotAclAuth(_) | MqttBrokerError::NotBlacklistAuth => {
                        (PubRecReason::NotAuthorized, PubAckReason::NotAuthorized)
                    }
                    _ => (
                        PubRecReason::UnspecifiedError,
                        PubAckReason::UnspecifiedError,
                    ),
                };
                return Some(build_pub_ack_fail(
                    &self.cache_manager,
                    connection.connect_id,
                    &self.protocol,
                    publish.p_kid,
                    (pub_rec_reason, pub_ack_reason, e.to_string()),
                    is_pub_ack,
                ));
            }
        };

        let user_properties: Vec<(String, String)> = vec![("offset".to_string(), offset)];

        self.cache_manager
            .add_topic_alias(connection.connect_id, &topic_name, publish_properties);

        record_publish_receive_metrics(
            &connection.client_id,
            connection.connect_id,
            &topic_name,
            publish.payload.len() as u64,
        );

        match publish.qos {
            QoS::AtMostOnce => None,
            QoS::AtLeastOnce => Some(build_pub_ack(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                publish.p_kid,
                PubAckReason::Success,
                None,
                user_properties,
            )),
            QoS::ExactlyOnce => {
                self.cache_manager
                    .pkid_metadata
                    .add_client_pkid(&connection.client_id, publish.p_kid);

                Some(build_pub_rec(
                    &self.cache_manager,
                    connection.connect_id,
                    &self.protocol,
                    publish.p_kid,
                    PubRecReason::Success,
                    None,
                    user_properties,
                ))
            }
        }
    }

    async fn process_publish0(
        &self,
        connection: &MQTTConnection,
        publish: &Publish,
        publish_properties: &Option<PublishProperties>,
    ) -> Result<(String, String), MqttBrokerError> {
        let mut topic_name = get_topic_name(
            &self.cache_manager,
            connection.connect_id,
            publish,
            publish_properties,
        )
        .await?;

        let mut delay_info = if is_delay_topic(&topic_name) {
            let data = decode_delay_topic(&topic_name)?;
            record_mqtt_messages_delayed_inc();
            topic_name = data.target_topic_name.clone();
            Some(data)
        } else {
            None
        };

        self.auth_driver
            .auth_publish_check(connection, &topic_name, publish.retain, publish.qos)
            .await?;

        let topic = try_init_topic(
            &topic_name,
            &self.cache_manager,
            &self.storage_driver_manager,
            &self.client_pool,
        )
        .await?;

        if delay_info.is_some() {
            let mut new_delay_info = delay_info.unwrap();
            new_delay_info.target_shard_name = Some(topic.topic_name.clone());
            delay_info = Some(new_delay_info);
        }

        if self.schema_manager.is_check_schema(&topic_name) {
            self.schema_manager
                .validate(&topic_name, &publish.payload)?;
        }

        let client_id = connection.client_id.clone();

        let offset = save_message(SaveMessageContext {
            storage_driver_manager: self.storage_driver_manager.clone(),
            delay_message_manager: self.delay_message_manager.clone(),
            cache_manager: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
            retain_message_manager: self.retain_message_manager.clone(),
            publish: publish.clone(),
            publish_properties: publish_properties.clone(),
            subscribe_manager: self.subscribe_manager.clone(),
            client_id: client_id.clone(),
            topic: topic.clone(),
            delay_info,
        })
        .await?;

        Ok((format!("{:?}", offset), topic_name))
    }
}

fn build_pub_ack_fail(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
    pkid: u16,
    reason_info: (PubRecReason, PubAckReason, String),
    is_pub_ack: bool,
) -> MqttPacket {
    debug!(
        connect_id = connect_id,
        pkid = pkid,
        protocol = ?protocol,
        pub_rec_reason = ?reason_info.0,
        pub_ack_reason = ?reason_info.1,
        reason_string = %reason_info.2,
        is_pub_ack = is_pub_ack,
        "Building publish acknowledgment failure packet"
    );

    if is_pub_ack {
        return build_pub_ack(
            cache_manager,
            connect_id,
            protocol,
            pkid,
            reason_info.1,
            Some(reason_info.2),
            Vec::new(),
        );
    }

    build_pub_rec(
        cache_manager,
        connect_id,
        protocol,
        pkid,
        reason_info.0,
        Some(reason_info.2),
        Vec::new(),
    )
}

fn build_pub_ack(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
    pkid: u16,
    reason: PubAckReason,
    reason_string: Option<String>,
    user_properties: Vec<(String, String)>,
) -> MqttPacket {
    let pub_ack = PubAck {
        pkid,
        reason: Some(reason),
    };

    if !protocol.is_mqtt5() {
        return MqttPacket::PubAck(pub_ack, None);
    }

    let mut properties = PubAckProperties {
        user_properties,
        ..Default::default()
    };
    if is_request_problem_info(cache_manager, connect_id) {
        properties.reason_string = reason_string;
    }
    MqttPacket::PubAck(pub_ack, Some(properties))
}

fn build_pub_rec(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
    pkid: u16,
    reason: PubRecReason,
    reason_string: Option<String>,
    user_properties: Vec<(String, String)>,
) -> MqttPacket {
    let pub_rec = PubRec {
        pkid,
        reason: Some(reason),
    };

    if !protocol.is_mqtt5() {
        return MqttPacket::PubRec(pub_rec, None);
    }

    let mut properties = PubRecProperties {
        user_properties,
        ..Default::default()
    };

    if is_request_problem_info(cache_manager, connect_id) {
        properties.reason_string = reason_string;
    }

    MqttPacket::PubRec(pub_rec, Some(properties))
}

async fn publish_validator(
    cache_manager: &Arc<MQTTCacheManager>,
    connection: &MQTTConnection,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Option<(PubRecReason, PubAckReason, String)> {
    if publish.qos == QoS::ExactlyOnce
        && cache_manager
            .pkid_metadata
            .get_client_pkid(&connection.client_id, publish.p_kid)
            .is_some()
    {
        return Some((
            PubRecReason::PacketIdentifierInUse,
            PubAckReason::PacketIdentifierInUse,
            format!("Packet identifier {} is already in use", publish.p_kid),
        ));
    }

    let cluster = cache_manager.broker_cache.get_cluster_config().await;

    let max_packet_size = min(
        cluster.mqtt_protocol_config.max_packet_size,
        connection.max_packet_size,
    ) as usize;
    if publish.payload.len() > max_packet_size {
        return Some((
            PubRecReason::PayloadFormatInvalid,
            PubAckReason::PayloadFormatInvalid,
            MqttBrokerError::PacketLengthError(max_packet_size, publish.payload.len()).to_string(),
        ));
    }

    if is_qos_message(publish.qos)
        && connection.get_recv_qos_message() >= cluster.mqtt_protocol_config.receive_max as isize
    {
        return Some((
            PubRecReason::QuotaExceeded,
            PubAckReason::QuotaExceeded,
            format!(
                "Receive maximum quota exceeded. Current: {}, Maximum: {}",
                connection.get_recv_qos_message(),
                cluster.mqtt_protocol_config.receive_max
            ),
        ));
    }

    if !payload_format_indicator_check_by_publish(publish, publish_properties) {
        return Some((
            PubRecReason::PayloadFormatInvalid,
            PubAckReason::PayloadFormatInvalid,
            MqttBrokerError::PayloadFormatInvalid.to_string(),
        ));
    }

    if let Some(properties) = publish_properties {
        if let Some(alias) = properties.topic_alias {
            if alias > connection.topic_alias_max {
                return Some((
                    PubRecReason::UnspecifiedError,
                    PubAckReason::UnspecifiedError,
                    MqttBrokerError::TopicAliasTooLong(alias).to_string(),
                ));
            }
        }
    }

    None
}
