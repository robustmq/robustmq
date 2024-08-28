// Copyright 2023 RobustMQ Team
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

use crate::handler::cache::{CacheManager, ConnectionLiveTime};
use crate::handler::cache::{QosAckPackageData, QosAckPackageType};
use crate::handler::connection::{build_connection, get_client_id};
use crate::handler::lastwill::save_last_will_message;
use crate::handler::pkid::{pkid_delete, pkid_exists, pkid_save};
use crate::handler::response::{
    response_packet_mqtt_connect_fail, response_packet_mqtt_connect_success,
    response_packet_mqtt_distinct, response_packet_mqtt_distinct_by_reason,
    response_packet_mqtt_ping_resp, response_packet_mqtt_puback_fail,
    response_packet_mqtt_puback_success, response_packet_mqtt_pubcomp_fail,
    response_packet_mqtt_pubcomp_success, response_packet_mqtt_pubrec_fail,
    response_packet_mqtt_pubrec_success, response_packet_mqtt_pubrel_success,
    response_packet_mqtt_suback, response_packet_mqtt_unsuback,
};
use crate::handler::retain::save_topic_retain_message;
use crate::handler::session::{build_session, save_session};
use crate::handler::topic::{get_topic_name, try_init_topic};
use crate::handler::validator::{
    connect_validator, publish_validator, subscribe_validator, un_subscribe_validator,
};
use crate::security::AuthDriver;
use crate::server::connection_manager::ConnectionManager;
use crate::storage::message::MessageStorage;
use crate::subscribe::sub_common::{min_qos, path_contain_sub};
use crate::subscribe::subscribe_manager::SubscribeManager;
use clients::poll::ClientPool;
use common_base::tools::now_second;
use log::error;
use metadata_struct::mqtt::message::MQTTMessage;
use protocol::mqtt::common::{
    Connect, ConnectProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, LastWill, LastWillProperties, Login, MQTTPacket, MQTTProtocol, PingReq,
    PubAck, PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec,
    PubRecProperties, PubRecReason, PubRel, PubRelProperties, PubRelReason, Publish,
    PublishProperties, QoS, Subscribe, SubscribeProperties, SubscribeReasonCode, UnsubAckReason,
    Unsubscribe, UnsubscribeProperties,
};
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

use super::connection::disconnect_connection;
use super::flow_control::is_flow_control;
use super::topic::topic_name_validator;

#[derive(Clone)]
pub struct MqttService<S> {
    protocol: MQTTProtocol,
    cache_manager: Arc<CacheManager>,
    connnection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    sucscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
    auth_driver: Arc<AuthDriver>,
}

impl<S> MqttService<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        protocol: MQTTProtocol,
        cache_manager: Arc<CacheManager>,
        connnection_manager: Arc<ConnectionManager>,
        message_storage_adapter: Arc<S>,
        sucscribe_manager: Arc<SubscribeManager>,
        client_poll: Arc<ClientPool>,
        auth_driver: Arc<AuthDriver>,
    ) -> Self {
        return MqttService {
            protocol,
            cache_manager,
            connnection_manager,
            message_storage_adapter,
            sucscribe_manager,
            client_poll,
            auth_driver,
        };
    }

    pub async fn connect(
        &mut self,
        connect_id: u64,
        connnect: Connect,
        connect_properties: Option<ConnectProperties>,
        last_will: Option<LastWill>,
        last_will_properties: Option<LastWillProperties>,
        login: Option<Login>,
        addr: SocketAddr,
    ) -> MQTTPacket {
        let cluster: metadata_struct::mqtt::cluster::MQTTCluster =
            self.cache_manager.get_cluster_info();

        if let Some(res) = connect_validator(
            &self.protocol,
            &cluster,
            &connnect,
            &connect_properties,
            &last_will,
            &last_will_properties,
            &login,
            &addr,
        ) {
            return res;
        }

        match self
            .auth_driver
            .check_login(&login, &connect_properties, &addr)
            .await
        {
            Ok(flag) => {
                if !flag {
                    return response_packet_mqtt_connect_fail(
                        &self.protocol,
                        ConnectReturnCode::NotAuthorized,
                        &connect_properties,
                        None,
                    );
                }
            }
            Err(e) => {
                return response_packet_mqtt_connect_fail(
                    &self.protocol,
                    ConnectReturnCode::UnspecifiedError,
                    &connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        let (client_id, new_client_id) = get_client_id(&connnect.client_id);

        let connection = build_connection(
            connect_id,
            &client_id,
            &cluster,
            &connnect,
            &connect_properties,
        );

        let (session, new_session) = match build_session(
            connect_id,
            &client_id,
            &connnect,
            &connect_properties,
            &last_will,
            &last_will_properties,
            &self.client_poll,
            &self.cache_manager,
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                return response_packet_mqtt_connect_fail(
                    &self.protocol,
                    ConnectReturnCode::MalformedPacket,
                    &connect_properties,
                    Some(e.to_string()),
                );
            }
        };

        match save_session(
            connect_id,
            session.clone(),
            new_session,
            &client_id,
            &self.client_poll,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                return response_packet_mqtt_connect_fail(
                    &self.protocol,
                    ConnectReturnCode::MalformedPacket,
                    &connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        match save_last_will_message(
            &client_id,
            &last_will,
            &last_will_properties,
            &self.client_poll,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                return response_packet_mqtt_connect_fail(
                    &self.protocol,
                    ConnectReturnCode::UnspecifiedError,
                    &connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        let live_time = ConnectionLiveTime {
            protobol: self.protocol.clone(),
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.cache_manager.report_heartbeat(&client_id, live_time);

        self.cache_manager
            .add_session(client_id.clone(), session.clone());
        self.cache_manager
            .add_connection(connect_id, connection.clone());

        return response_packet_mqtt_connect_success(
            &self.protocol,
            &cluster,
            client_id.clone(),
            new_client_id,
            session.session_expiry as u32,
            new_session,
            connection.keep_alive,
            &connect_properties,
        );
    }

    pub async fn publish(
        &self,
        connect_id: u64,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> Option<MQTTPacket> {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return Some(response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
            ));
        };

        if is_flow_control(&self.protocol, publish.qos) {
            connection.recv_qos_message_incr();
        }

        if let Some(pkg) = publish_validator(
            &self.protocol,
            &self.cache_manager,
            &self.client_poll,
            &connection,
            &publish,
            &publish_properties,
        )
        .await
        {
            if is_flow_control(&self.protocol, publish.qos) {
                connection.recv_qos_message_decr();
            }
            if publish.qos == QoS::AtMostOnce {
                return None;
            } else {
                return Some(pkg);
            }
        }

        let is_puback = publish.qos != QoS::ExactlyOnce;

        let topic_name = match get_topic_name(
            connect_id,
            &self.cache_manager,
            &publish,
            &publish_properties,
        ) {
            Ok(da) => da,
            Err(e) => {
                if is_flow_control(&self.protocol, publish.qos) {
                    connection.recv_qos_message_decr();
                }

                if is_puback {
                    return Some(response_packet_mqtt_puback_fail(
                        &self.protocol,
                        &connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                } else {
                    return Some(response_packet_mqtt_pubrec_fail(
                        &self.protocol,
                        &connection,
                        publish.pkid,
                        PubRecReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                }
            }
        };

        let topic = match try_init_topic(
            &topic_name,
            &self.cache_manager,
            &self.message_storage_adapter,
            &self.client_poll,
        )
        .await
        {
            Ok(tp) => tp,
            Err(e) => {
                if is_flow_control(&self.protocol, publish.qos) {
                    connection.recv_qos_message_decr();
                }

                if is_puback {
                    return Some(response_packet_mqtt_puback_fail(
                        &self.protocol,
                        &connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                } else {
                    return Some(response_packet_mqtt_pubrec_fail(
                        &self.protocol,
                        &connection,
                        publish.pkid,
                        PubRecReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                }
            }
        };

        let client_id = connection.client_id.clone();

        // Persisting retain message data
        match save_topic_retain_message(
            &self.cache_manager,
            &self.client_poll,
            &topic_name,
            &client_id,
            &publish,
            &publish_properties,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                if is_flow_control(&self.protocol, publish.qos) {
                    connection.recv_qos_message_decr();
                }

                if is_puback {
                    return Some(response_packet_mqtt_puback_fail(
                        &self.protocol,
                        &connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                } else {
                    return Some(response_packet_mqtt_pubrec_fail(
                        &self.protocol,
                        &connection,
                        publish.pkid,
                        PubRecReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                }
            }
        }

        // Persisting stores message data
        let message_storage = MessageStorage::new(self.message_storage_adapter.clone());
        let offset = if let Some(record) =
            MQTTMessage::build_record(&client_id, &publish, &publish_properties)
        {
            match message_storage
                .append_topic_message(topic.topic_id.clone(), vec![record])
                .await
            {
                Ok(da) => {
                    format!("{:?}", da)
                }
                Err(e) => {
                    if is_flow_control(&self.protocol, publish.qos) {
                        connection.recv_qos_message_decr();
                    }

                    if is_puback {
                        return Some(response_packet_mqtt_puback_fail(
                            &self.protocol,
                            &connection,
                            publish.pkid,
                            PubAckReason::UnspecifiedError,
                            Some(e.to_string()),
                        ));
                    } else {
                        return Some(response_packet_mqtt_pubrec_fail(
                            &self.protocol,
                            &connection,
                            publish.pkid,
                            PubRecReason::UnspecifiedError,
                            Some(e.to_string()),
                        ));
                    }
                }
            }
        } else {
            "-1".to_string()
        };
        let user_properties: Vec<(String, String)> = vec![("offset".to_string(), offset)];

        self.cache_manager
            .add_topic_alias(connect_id, &topic_name, &publish_properties);

        match publish.qos {
            QoS::AtMostOnce => {
                return None;
            }
            QoS::AtLeastOnce => {
                if is_flow_control(&self.protocol, publish.qos) {
                    connection.recv_qos_message_decr();
                }

                let reason_code = if path_contain_sub(&topic_name) {
                    PubAckReason::Success
                } else {
                    PubAckReason::NoMatchingSubscribers
                };
                return Some(response_packet_mqtt_puback_success(
                    &self.protocol,
                    reason_code,
                    publish.pkid,
                    user_properties,
                ));
            }
            QoS::ExactlyOnce => {
                match pkid_save(
                    &self.cache_manager,
                    &self.client_poll,
                    &client_id,
                    publish.pkid,
                )
                .await
                {
                    Ok(()) => {}
                    Err(e) => {
                        if is_flow_control(&self.protocol, publish.qos) {
                            connection.recv_qos_message_decr();
                        }

                        if is_puback {
                            return Some(response_packet_mqtt_puback_fail(
                                &self.protocol,
                                &connection,
                                publish.pkid,
                                PubAckReason::UnspecifiedError,
                                Some(e.to_string()),
                            ));
                        } else {
                            return Some(response_packet_mqtt_pubrec_fail(
                                &self.protocol,
                                &connection,
                                publish.pkid,
                                PubRecReason::UnspecifiedError,
                                Some(e.to_string()),
                            ));
                        }
                    }
                }
                let reason_code = if path_contain_sub(&topic_name) {
                    PubRecReason::Success
                } else {
                    PubRecReason::NoMatchingSubscribers
                };

                return Some(response_packet_mqtt_pubrec_success(
                    &self.protocol,
                    reason_code,
                    publish.pkid,
                    user_properties,
                ));
            }
        }
    }

    pub async fn publish_ack(
        &self,
        connect_id: u64,
        pub_ack: PubAck,
        _: Option<PubAckProperties>,
    ) -> Option<MQTTPacket> {
        if let Some(conn) = self.cache_manager.connection_info.get(&connect_id) {
            let client_id = conn.client_id.clone();
            let pkid = pub_ack.pkid;
            if let Some(data) = self.cache_manager.get_ack_packet(client_id.clone(), pkid) {
                match data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubAck,
                    pkid: pub_ack.pkid,
                }) {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "publish ack send ack manager message error, error message:{}",
                            e
                        );
                    }
                }
            }
        }

        return None;
    }

    pub async fn publish_rec(
        &self,
        connect_id: u64,
        pub_rec: PubRec,
        _: Option<PubRecProperties>,
    ) -> Option<MQTTPacket> {
        if let Some(conn) = self.cache_manager.connection_info.get(&connect_id) {
            let client_id = conn.client_id.clone();
            let pkid = pub_rec.pkid;
            if let Some(data) = self.cache_manager.get_ack_packet(client_id.clone(), pkid) {
                match data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubRec,
                    pkid: pub_rec.pkid,
                }) {
                    Ok(_) => return None,
                    Err(e) => {
                        error!(
                            "publish rec send ack manager message error, error message:{}",
                            e
                        );
                    }
                }
            }
        }

        return Some(response_packet_mqtt_pubrel_success(
            &self.protocol,
            pub_rec.pkid,
            PubRelReason::Success,
        ));
    }

    pub async fn publish_comp(
        &self,
        connect_id: u64,
        pub_comp: PubComp,
        _: Option<PubCompProperties>,
    ) -> Option<MQTTPacket> {
        if let Some(conn) = self.cache_manager.connection_info.get(&connect_id) {
            let client_id = conn.client_id.clone();
            let pkid = pub_comp.pkid;
            if let Some(data) = self.cache_manager.get_ack_packet(client_id.clone(), pkid) {
                match data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubComp,
                    pkid: pub_comp.pkid,
                }) {
                    Ok(_) => return None,
                    Err(e) => {
                        error!(
                            "publish comp send ack manager message error, error message:{}",
                            e
                        );
                    }
                }
            }
        }
        return None;
    }

    pub async fn publish_rel(
        &self,
        connect_id: u64,
        pub_rel: PubRel,
        _: Option<PubRelProperties>,
    ) -> MQTTPacket {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
            );
        };

        let client_id = connection.client_id.clone();

        match pkid_exists(
            &self.cache_manager,
            &self.client_poll,
            &client_id,
            pub_rel.pkid,
        )
        .await
        {
            Ok(res) => {
                if !res {
                    return response_packet_mqtt_pubcomp_fail(
                        &self.protocol,
                        &connection,
                        pub_rel.pkid,
                        PubCompReason::PacketIdentifierNotFound,
                        None,
                    );
                }
            }
            Err(e) => {
                return response_packet_mqtt_pubcomp_fail(
                    &self.protocol,
                    &connection,
                    pub_rel.pkid,
                    PubCompReason::PacketIdentifierNotFound,
                    Some(e.to_string()),
                );
            }
        };

        match pkid_delete(
            &self.cache_manager,
            &self.client_poll,
            &client_id,
            pub_rel.pkid,
        )
        .await
        {
            Ok(()) => {
                connection.recv_qos_message_decr();
            }
            Err(e) => {
                return response_packet_mqtt_pubcomp_fail(
                    &self.protocol,
                    &connection,
                    pub_rel.pkid,
                    PubCompReason::PacketIdentifierNotFound,
                    Some(e.to_string()),
                );
            }
        }
        return response_packet_mqtt_pubcomp_success(&self.protocol, pub_rel.pkid);
    }

    pub async fn subscribe(
        &self,
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) -> MQTTPacket {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
            );
        };

        let client_id = connection.client_id.clone();

        if let Some(packet) = subscribe_validator(
            &self.protocol,
            &self.cache_manager,
            &self.client_poll,
            &connection,
            &subscribe,
        )
        .await
        {
            return packet;
        }

        let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
        let cluster_qos = self.cache_manager.get_cluster_info().max_qos();
        for filter in subscribe.filters.clone() {
            match min_qos(cluster_qos, filter.qos) {
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

        match pkid_save(
            &self.cache_manager,
            &self.client_poll,
            &client_id,
            subscribe.packet_identifier,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                return response_packet_mqtt_suback(
                    &self.protocol,
                    &connection,
                    subscribe.packet_identifier,
                    vec![SubscribeReasonCode::Unspecified],
                    Some(e.to_string()),
                );
            }
        }

        self.cache_manager.add_client_subscribe(
            client_id.clone(),
            self.protocol.clone(),
            subscribe.clone(),
            subscribe_properties.clone(),
        );

        self.sucscribe_manager
            .add_subscribe(
                client_id.clone(),
                self.protocol.clone(),
                subscribe.clone(),
                subscribe_properties.clone(),
            )
            .await;

        let pkid = subscribe.packet_identifier;
        return response_packet_mqtt_suback(&self.protocol, &connection, pkid, return_codes, None);
    }

    pub async fn ping(&self, connect_id: u64, _: PingReq) -> MQTTPacket {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
            );
        };

        let live_time = ConnectionLiveTime {
            protobol: self.protocol.clone(),
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.cache_manager
            .report_heartbeat(&connection.client_id, live_time);
        return response_packet_mqtt_ping_resp();
    }

    pub async fn un_subscribe(
        &self,
        connect_id: u64,
        un_subscribe: Unsubscribe,
        _: Option<UnsubscribeProperties>,
    ) -> MQTTPacket {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
            );
        };

        if let Some(packet) = un_subscribe_validator(
            &connection.client_id,
            &self.cache_manager,
            &self.client_poll,
            &connection,
            &un_subscribe,
        )
        .await
        {
            return packet;
        }

        match pkid_delete(
            &self.cache_manager,
            &self.client_poll,
            &connection.client_id,
            un_subscribe.pkid,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                return response_packet_mqtt_unsuback(
                    &connection,
                    un_subscribe.pkid,
                    vec![UnsubAckReason::UnspecifiedError],
                    Some(e.to_string()),
                );
            }
        }

        self.sucscribe_manager
            .remove_subscribe(&connection.client_id, &un_subscribe.filters);

        self.cache_manager
            .remove_filter_by_pkid(&connection.client_id, &un_subscribe.filters);

        return response_packet_mqtt_unsuback(
            &connection,
            un_subscribe.pkid,
            vec![UnsubAckReason::Success],
            None,
        );
    }

    pub async fn disconnect(
        &self,
        connect_id: u64,
        _: Disconnect,
        _: Option<DisconnectProperties>,
    ) -> Option<MQTTPacket> {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return None;
        };

        match disconnect_connection(
            &connection.client_id,
            connect_id,
            &self.cache_manager,
            &self.client_poll,
            &self.connnection_manager,
            &self.sucscribe_manager,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                return Some(response_packet_mqtt_distinct(
                    &self.protocol,
                    Some(DisconnectReasonCode::UnspecifiedError),
                    &connection,
                    Some(e.to_string()),
                ));
            }
        }

        return None;
    }
}
