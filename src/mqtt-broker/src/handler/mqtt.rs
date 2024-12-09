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

use std::net::SocketAddr;
use std::sync::Arc;

use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use log::{error, warn};
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{
    Connect, ConnectProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, LastWill, LastWillProperties, Login, MqttPacket, MqttProtocol, PingReq,
    PubAck, PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec,
    PubRecProperties, PubRecReason, PubRel, PubRelProperties, PubRelReason, Publish,
    PublishProperties, QoS, Subscribe, SubscribeProperties, SubscribeReasonCode, UnsubAckReason,
    Unsubscribe, UnsubscribeProperties,
};
use storage_adapter::storage::StorageAdapter;

use super::connection::disconnect_connection;
use super::flow_control::is_flow_control;
use super::message::build_message_expire;
use super::retain::try_send_retain_message;
use crate::handler::cache::{
    CacheManager, ConnectionLiveTime, QosAckPackageData, QosAckPackageType,
};
use crate::handler::connection::{build_connection, get_client_id};
use crate::handler::lastwill::save_last_will_message;
use crate::handler::pkid::{pkid_delete, pkid_exists, pkid_save};
use crate::handler::response::{
    response_packet_mqtt_connect_fail, response_packet_mqtt_connect_success,
    response_packet_mqtt_distinct_by_reason, response_packet_mqtt_ping_resp,
    response_packet_mqtt_puback_fail, response_packet_mqtt_puback_success,
    response_packet_mqtt_pubcomp_fail, response_packet_mqtt_pubcomp_success,
    response_packet_mqtt_pubrec_fail, response_packet_mqtt_pubrec_success,
    response_packet_mqtt_pubrel_success, response_packet_mqtt_suback,
    response_packet_mqtt_unsuback,
};
use crate::handler::retain::save_retain_message;
use crate::handler::session::{build_session, save_session};
use crate::handler::topic::{get_topic_name, try_init_topic};
use crate::handler::validator::{
    connect_validator, publish_validator, subscribe_validator, un_subscribe_validator,
};
use crate::observability::system_topic::event::{
    st_report_connected_event, st_report_disconnected_event, st_report_subscribed_event,
    st_report_unsubscribed_event,
};
use crate::security::AuthDriver;
use crate::server::connection_manager::ConnectionManager;
use crate::storage::message::MessageStorage;
use crate::subscribe::sub_common::{min_qos, path_contain_sub};
use crate::subscribe::subscribe_manager::SubscribeManager;

#[derive(Clone)]
pub struct MqttService<S> {
    protocol: MqttProtocol,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    subscribe_manager: Arc<SubscribeManager>,
    client_pool: Arc<ClientPool>,
    auth_driver: Arc<AuthDriver>,
}

impl<S> MqttService<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        protocol: MqttProtocol,
        cache_manager: Arc<CacheManager>,
        connection_manager: Arc<ConnectionManager>,
        message_storage_adapter: Arc<S>,
        subscribe_manager: Arc<SubscribeManager>,
        client_pool: Arc<ClientPool>,
        auth_driver: Arc<AuthDriver>,
    ) -> Self {
        MqttService {
            protocol,
            cache_manager,
            connection_manager,
            message_storage_adapter,
            subscribe_manager,
            client_pool,
            auth_driver,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn connect(
        &mut self,
        connect_id: u64,
        connect: Connect,
        connect_properties: Option<ConnectProperties>,
        last_will: Option<LastWill>,
        last_will_properties: Option<LastWillProperties>,
        login: &Option<Login>,
        addr: SocketAddr,
    ) -> MqttPacket {
        let cluster: metadata_struct::mqtt::cluster::MqttClusterDynamicConfig =
            self.cache_manager.get_cluster_info();

        if let Some(res) = connect_validator(
            &self.protocol,
            &cluster,
            &connect,
            &connect_properties,
            &last_will,
            &last_will_properties,
            login,
            &addr,
        ) {
            return res;
        }

        match self
            .auth_driver
            .check_login_auth(login, &connect_properties, &addr)
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

        let (client_id, new_client_id) = get_client_id(&connect.client_id);

        let connection = build_connection(
            connect_id,
            client_id.clone(),
            &cluster,
            &connect,
            &connect_properties,
            &addr,
        );

        let (session, new_session) = match build_session(
            connect_id,
            client_id.clone(),
            &connect,
            &connect_properties,
            &last_will,
            &last_will_properties,
            &self.client_pool,
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
            client_id.clone(),
            &self.client_pool,
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
            client_id.clone(),
            &last_will,
            &last_will_properties,
            &self.client_pool,
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
        self.cache_manager
            .report_heartbeat(client_id.clone(), live_time);

        self.cache_manager
            .add_session(client_id.clone(), session.clone());
        self.cache_manager
            .add_connection(connect_id, connection.clone());

        st_report_connected_event(
            &self.message_storage_adapter,
            &self.cache_manager,
            &self.client_pool,
            &session,
            &connection,
            connect_id,
            &self.connection_manager,
        )
        .await;

        response_packet_mqtt_connect_success(
            &self.protocol,
            &cluster,
            client_id,
            new_client_id,
            session.session_expiry as u32,
            new_session,
            connection.keep_alive,
            &connect_properties,
        )
    }

    pub async fn publish(
        &self,
        connect_id: u64,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> Option<MqttPacket> {
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
            &self.client_pool,
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

        if !self
            .auth_driver
            .allow_publish(&connection, &topic_name, publish.retain, publish.qos)
            .await
        {
            if is_puback {
                return Some(response_packet_mqtt_puback_fail(
                    &self.protocol,
                    &connection,
                    publish.pkid,
                    PubAckReason::NotAuthorized,
                    None,
                ));
            } else {
                return Some(response_packet_mqtt_pubrec_fail(
                    &self.protocol,
                    &connection,
                    publish.pkid,
                    PubRecReason::NotAuthorized,
                    None,
                ));
            }
        }

        let topic = match try_init_topic(
            &topic_name,
            &self.cache_manager,
            &self.message_storage_adapter,
            &self.client_pool,
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
        match save_retain_message(
            &self.cache_manager,
            &self.client_pool,
            topic_name.clone(),
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

        let message_expire = build_message_expire(&self.cache_manager, &publish_properties);
        let offset = if let Some(record) =
            MqttMessage::build_record(&client_id, &publish, &publish_properties, message_expire)
        {
            match message_storage
                .append_topic_message(&topic.topic_id, vec![record])
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
            QoS::AtMostOnce => None,
            QoS::AtLeastOnce => {
                if is_flow_control(&self.protocol, publish.qos) {
                    connection.recv_qos_message_decr();
                }

                let reason_code = if path_contain_sub(&topic_name) {
                    PubAckReason::Success
                } else {
                    PubAckReason::NoMatchingSubscribers
                };
                Some(response_packet_mqtt_puback_success(
                    &self.protocol,
                    reason_code,
                    publish.pkid,
                    user_properties,
                ))
            }
            QoS::ExactlyOnce => {
                match pkid_save(
                    &self.cache_manager,
                    &self.client_pool,
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

                Some(response_packet_mqtt_pubrec_success(
                    &self.protocol,
                    reason_code,
                    publish.pkid,
                    user_properties,
                ))
            }
        }
    }

    pub async fn publish_ack(
        &self,
        connect_id: u64,
        pub_ack: PubAck,
        _: Option<PubAckProperties>,
    ) -> Option<MqttPacket> {
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

        None
    }

    pub async fn publish_rec(
        &self,
        connect_id: u64,
        pub_rec: PubRec,
        _: Option<PubRecProperties>,
    ) -> Option<MqttPacket> {
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

        Some(response_packet_mqtt_pubrel_success(
            &self.protocol,
            pub_rec.pkid,
            PubRelReason::Success,
        ))
    }

    pub async fn publish_comp(
        &self,
        connect_id: u64,
        pub_comp: PubComp,
        _: Option<PubCompProperties>,
    ) -> Option<MqttPacket> {
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
        None
    }

    pub async fn publish_rel(
        &self,
        connect_id: u64,
        pub_rel: PubRel,
        _: Option<PubRelProperties>,
    ) -> MqttPacket {
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
            &self.client_pool,
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
            &self.client_pool,
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
        response_packet_mqtt_pubcomp_success(&self.protocol, pub_rel.pkid)
    }

    pub async fn subscribe(
        &self,
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) -> MqttPacket {
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
            &self.client_pool,
            &connection,
            &subscribe,
        )
        .await
        {
            return packet;
        }

        if !self
            .auth_driver
            .allow_subscribe(&connection, &subscribe)
            .await
        {
            return response_packet_mqtt_suback(
                &self.protocol,
                &connection,
                subscribe.packet_identifier,
                vec![SubscribeReasonCode::NotAuthorized],
                None,
            );
        }

        let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
        let cluster_qos = self.cache_manager.get_cluster_info().protocol.max_qos;
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

        // match pkid_save(
        //     &self.cache_manager,
        //     &self.client_pool,
        //     &client_id,
        //     subscribe.packet_identifier,
        // )
        // .await
        // {
        //     Ok(()) => {}
        //     Err(e) => {
        //         return response_packet_mqtt_suback(
        //             &self.protocol,
        //             &connection,
        //             subscribe.packet_identifier,
        //             vec![SubscribeReasonCode::Unspecified],
        //             Some(e.to_string()),
        //         );
        //     }
        // }

        match self
            .subscribe_manager
            .save_exclusive_subscribe(subscribe.clone())
            .await
        {
            Ok(None) => {}
            Ok(Some(code)) => {
                return response_packet_mqtt_suback(
                    &self.protocol,
                    &connection,
                    subscribe.packet_identifier,
                    vec![code],
                    None,
                );
            }
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

        self.subscribe_manager
            .add_subscribe(
                client_id.clone(),
                self.protocol.clone(),
                subscribe.clone(),
                subscribe_properties.clone(),
            )
            .await;

        let pkid = subscribe.packet_identifier;

        st_report_subscribed_event(
            &self.message_storage_adapter,
            &self.cache_manager,
            &self.client_pool,
            &connection,
            connect_id,
            &self.connection_manager,
            &subscribe,
        )
        .await;

        try_send_retain_message(
            self.protocol.clone(),
            client_id.clone(),
            subscribe.clone(),
            subscribe_properties.clone(),
            self.client_pool.clone(),
            self.cache_manager.clone(),
            self.connection_manager.clone(),
        )
        .await;

        response_packet_mqtt_suback(&self.protocol, &connection, pkid, return_codes, None)
    }

    pub async fn ping(&self, connect_id: u64, _: PingReq) -> MqttPacket {
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
            .report_heartbeat(connection.client_id, live_time);
        response_packet_mqtt_ping_resp()
    }

    pub async fn un_subscribe(
        &self,
        connect_id: u64,
        un_subscribe: Unsubscribe,
        _: Option<UnsubscribeProperties>,
    ) -> MqttPacket {
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
            &self.client_pool,
            &connection,
            &un_subscribe,
        )
        .await
        {
            return packet;
        }

        // match pkid_delete(
        //     &self.cache_manager,
        //     &self.client_pool,
        //     &connection.client_id,
        //     un_subscribe.pkid,
        // )
        // .await
        // {
        //     Ok(()) => {}
        //     Err(e) => {
        //         return response_packet_mqtt_unsuback(
        //             &connection,
        //             un_subscribe.pkid,
        //             vec![UnsubAckReason::UnspecifiedError],
        //             Some(e.to_string()),
        //         );
        //     }
        // }

        match self
            .subscribe_manager
            .remove_exclusive_subscribe(un_subscribe.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return response_packet_mqtt_suback(
                    &self.protocol,
                    &connection,
                    un_subscribe.pkid,
                    vec![SubscribeReasonCode::Unspecified],
                    Some(e.to_string()),
                );
            }
        }

        self.subscribe_manager
            .remove_subscribe(&connection.client_id, &un_subscribe.filters);

        self.cache_manager
            .remove_filter_by_pkid(&connection.client_id, &un_subscribe.filters);

        st_report_unsubscribed_event(
            &self.message_storage_adapter,
            &self.cache_manager,
            &self.client_pool,
            &connection,
            connect_id,
            &self.connection_manager,
            &un_subscribe,
        )
        .await;

        response_packet_mqtt_unsuback(
            &connection,
            un_subscribe.pkid,
            vec![UnsubAckReason::Success],
            None,
        )
    }

    pub async fn disconnect(
        &self,
        connect_id: u64,
        disconnect: Disconnect,
        _: Option<DisconnectProperties>,
    ) -> Option<MqttPacket> {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return None;
        };

        if let Some(session) = self.cache_manager.get_session_info(&connection.client_id) {
            st_report_disconnected_event(
                &self.message_storage_adapter,
                &self.cache_manager,
                &self.client_pool,
                &session,
                &connection,
                connect_id,
                &self.connection_manager,
                disconnect.reason_code,
            )
            .await;
        }

        match disconnect_connection(
            &connection.client_id,
            connect_id,
            &self.cache_manager,
            &self.client_pool,
            &self.connection_manager,
            &self.subscribe_manager,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                warn!("disconnect connection failed, {}", e.to_string());
            }
        }

        None
    }
}
