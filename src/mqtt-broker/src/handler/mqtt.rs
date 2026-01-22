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

use super::connection::{disconnect_connection, is_delete_session};
use super::delay_message::{decode_delay_topic, is_delay_topic};
use super::offline_message::{save_message, SaveMessageContext};
use super::response::build_pub_ack_fail;
use super::retain::{is_new_sub, try_send_retain_message, TrySendRetainMessageContext};
use super::sub_auto::try_auto_subscribe;
use super::subscribe::{save_subscribe, SaveSubscribeContext};
use crate::handler::cache::{
    ConnectionLiveTime, MQTTCacheManager, QosAckPackageData, QosAckPackageType,
};
use crate::handler::connection::{build_connection, get_client_id};
use crate::handler::flapping_detect::check_flapping_detect;
use crate::handler::last_will::save_last_will_message;
use crate::handler::metrics::record_publish_receive_metrics;
use crate::handler::response::{
    build_puback, build_pubrec, response_packet_mqtt_connect_fail,
    response_packet_mqtt_connect_success, response_packet_mqtt_distinct_by_reason,
    response_packet_mqtt_ping_resp, response_packet_mqtt_pubcomp_fail,
    response_packet_mqtt_pubcomp_success, response_packet_mqtt_suback,
    response_packet_mqtt_unsuback, ResponsePacketMqttConnectSuccessContext,
};
use crate::handler::session::{build_session, save_session, BuildSessionContext};
use crate::handler::subscribe::remove_subscribe;
use crate::handler::topic::{get_topic_name, try_init_topic};
use crate::handler::validator::{
    connect_validator, publish_validator, subscribe_validator, un_subscribe_validator,
};
use crate::security::AuthDriver;
use crate::subscribe::common::min_qos;
use crate::subscribe::manager::SubscribeManager;
use crate::system_topic::event::{
    st_report_connected_event, st_report_disconnected_event, st_report_subscribed_event,
    st_report_unsubscribed_event, StReportConnectedEventContext, StReportDisconnectedEventContext,
    StReportSubscribedEventContext, StReportUnsubscribedEventContext,
};
use common_base::tools::{now_millis, now_second};
use common_metrics::mqtt::auth::{record_mqtt_auth_failed, record_mqtt_auth_success};
use common_metrics::mqtt::publish::record_mqtt_messages_delayed_inc;
use delay_message::manager::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{
    qos, Connect, ConnectProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, LastWill, LastWillProperties, Login, MqttPacket, MqttProtocol, PingReq,
    PubAck, PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec,
    PubRecProperties, PubRecReason, PubRel, PubRelProperties, Publish, PublishProperties, QoS,
    Subscribe, SubscribeProperties, SubscribeReasonCode, UnsubAckReason, Unsubscribe,
    UnsubscribeProperties,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use schema_register::schema::SchemaRegisterManager;
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct MqttService {
    protocol: MqttProtocol,
    cache_manager: Arc<MQTTCacheManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    delay_message_manager: Arc<DelayMessageManager>,
    subscribe_manager: Arc<SubscribeManager>,
    schema_manager: Arc<SchemaRegisterManager>,
    client_pool: Arc<ClientPool>,
    auth_driver: Arc<AuthDriver>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

#[derive(Clone)]
pub struct MqttServiceContext {
    pub protocol: MqttProtocol,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub client_pool: Arc<ClientPool>,
    pub auth_driver: Arc<AuthDriver>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}

#[derive(Clone)]
pub struct MqttServiceConnectContext {
    pub connect_id: u64,
    pub connect: Connect,
    pub connect_properties: Option<ConnectProperties>,
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
    pub login: Option<Login>,
    pub addr: SocketAddr,
}

impl MqttService {
    pub fn new(context: MqttServiceContext) -> Self {
        MqttService {
            protocol: context.protocol,
            cache_manager: context.cache_manager,
            connection_manager: context.connection_manager,
            storage_driver_manager: context.storage_driver_manager,
            delay_message_manager: context.delay_message_manager,
            subscribe_manager: context.subscribe_manager,
            client_pool: context.client_pool,
            auth_driver: context.auth_driver,
            schema_manager: context.schema_manager,
            rocksdb_engine_handler: context.rocksdb_engine_handler,
        }
    }

    pub async fn connect(&self, context: MqttServiceConnectContext) -> MqttPacket {
        let cluster = self.cache_manager.broker_cache.get_cluster_config().await;

        if let Some(res) = connect_validator(
            &self.protocol,
            &cluster,
            &context.connect,
            &context.connect_properties,
            &context.last_will,
            &context.last_will_properties,
            &context.login,
        ) {
            return res;
        }


        let (data, resp) = get_client_id(
            &self.protocol,
            context.connect.clean_session,
            &context.connect.client_id,
        );

        if let Some(pkt) = resp {
            return pkt;
        }

        let Some((client_id, new_client_id)) = data else {
            return response_packet_mqtt_connect_fail(
                &self.protocol,
                ConnectReturnCode::UnspecifiedError,
                &context.connect_properties,
                Some("get_client_id returned empty result".to_string()),
            );
        };

        let connection = build_connection(
            context.connect_id,
            client_id.clone(),
            &self.cache_manager,
            &context.connect,
            &context.connect_properties,
            &context.addr,
        )
        .await;

        if self.auth_driver.auth_connect_check(&connection).await {
            return response_packet_mqtt_connect_fail(
                &self.protocol,
                ConnectReturnCode::Banned,
                &context.connect_properties,
                None,
            );
        }

        // login check
        match self
            .auth_driver
            .auth_login_check(
                &context.login,
                &context.connect_properties,
                &context.addr,
                Some(&context.connect.client_id),
            )
            .await
        {
            Ok(flag) => {
                if !flag {
                    record_mqtt_auth_failed();
                    return response_packet_mqtt_connect_fail(
                        &self.protocol,
                        ConnectReturnCode::NotAuthorized,
                        &context.connect_properties,
                        None,
                    );
                }
                record_mqtt_auth_success();
            }
            Err(e) => {
                return response_packet_mqtt_connect_fail(
                    &self.protocol,
                    ConnectReturnCode::UnspecifiedError,
                    &context.connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        // flapping detect check
        if cluster.mqtt_flapping_detect.enable {
            if let Err(e) = check_flapping_detect(
                context.connect.client_id.clone(),
                &self.cache_manager,
                &self.rocksdb_engine_handler,
            )
            .await
            {
                return response_packet_mqtt_connect_fail(
                    &self.protocol,
                    ConnectReturnCode::UnspecifiedError,
                    &context.connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        let (session, new_session) = match build_session(BuildSessionContext {
            connect_id: context.connect_id,
            client_id: client_id.clone(),
            connect: context.connect.clone(),
            connect_properties: context.connect_properties.clone(),
            last_will: context.last_will.clone(),
            last_will_properties: context.last_will_properties.clone(),
            client_pool: self.client_pool.clone(),
            cache_manager: self.cache_manager.clone(),
        })
        .await
        {
            Ok(data) => data,
            Err(e) => {
                return response_packet_mqtt_connect_fail(
                    &self.protocol,
                    ConnectReturnCode::MalformedPacket,
                    &context.connect_properties,
                    Some(e.to_string()),
                );
            }
        };

        if let Err(e) = save_session(
            context.connect_id,
            session.clone(),
            new_session,
            client_id.clone(),
            &self.client_pool,
        )
        .await
        {
            return response_packet_mqtt_connect_fail(
                &self.protocol,
                ConnectReturnCode::MalformedPacket,
                &context.connect_properties,
                Some(e.to_string()),
            );
        }

        if let Err(e) = save_last_will_message(
            client_id.clone(),
            &context.last_will,
            &context.last_will_properties,
            &self.client_pool,
        )
        .await
        {
            return response_packet_mqtt_connect_fail(
                &self.protocol,
                ConnectReturnCode::UnspecifiedError,
                &context.connect_properties,
                Some(e.to_string()),
            );
        }

        if let Err(e) = try_auto_subscribe(
            client_id.clone(),
            &context.login,
            &self.protocol,
            &self.client_pool,
            &self.cache_manager,
            &self.subscribe_manager,
        )
        .await
        {
            return response_packet_mqtt_connect_fail(
                &self.protocol,
                ConnectReturnCode::UnspecifiedError,
                &context.connect_properties,
                Some(e.to_string()),
            );
        }

        let live_time = ConnectionLiveTime {
            protocol: self.protocol.clone(),
            keep_live: context.connect.keep_alive,
            heartbeat: now_second(),
        };
        self.cache_manager
            .report_heartbeat(client_id.clone(), live_time);

        self.cache_manager.add_session(&client_id, &session);
        self.cache_manager
            .add_connection(context.connect_id, connection.clone());
        st_report_connected_event(StReportConnectedEventContext {
            storage_driver_manager: self.storage_driver_manager.clone(),
            metadata_cache: self.cache_manager.clone(),
            client_pool: self.client_pool.clone(),
            session: session.clone(),
            connection: connection.clone(),
            connect_id: context.connect_id,
            connection_manager: self.connection_manager.clone(),
        })
        .await;
        response_packet_mqtt_connect_success(ResponsePacketMqttConnectSuccessContext {
            protocol: self.protocol.clone(),
            cluster: cluster.clone(),
            client_id: client_id.clone(),
            auto_client_id: new_client_id,
            session_expiry_interval: session.session_expiry as u32,
            session_present: new_session,
            keep_alive: connection.keep_alive,
            connect_properties: context.connect_properties.clone(),
        })
    }

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

    pub async fn publish_ack(
        &self,
        connect_id: u64,
        pub_ack: &PubAck,
        _: &Option<PubAckProperties>,
    ) -> Option<MqttPacket> {
        if let Some(conn) = self.cache_manager.get_connection(connect_id) {
            let client_id = conn.client_id.clone();
            let pkid = pub_ack.pkid;
            if let Some(data) = self
                .cache_manager
                .pkid_metadata
                .get_ack_packet(&client_id, pkid)
            {
                if let Err(e) = data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubAck,
                    pkid: pub_ack.pkid,
                }) {
                    debug!(
                            "send puback to channel fail, error message:{}, send data time: {}, recv ack time:{}, client_id: {}, pkid: {}, connect_id:{}, diff:{}ms",
                            e,data.create_time, now_millis(), conn.client_id, pub_ack.pkid, connect_id, now_millis() -  data.create_time
                        );
                }
            }
        }

        None
    }

    pub async fn publish_rec(
        &self,
        connect_id: u64,
        pub_rec: &PubRec,
        _: &Option<PubRecProperties>,
    ) -> Option<MqttPacket> {
        if let Some(conn) = self.cache_manager.get_connection(connect_id) {
            let client_id = conn.client_id;
            let pkid = pub_rec.pkid;
            if let Some(data) = self
                .cache_manager
                .pkid_metadata
                .get_ack_packet(&client_id, pkid)
            {
                if let Err(e) = data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubRec,
                    pkid: pub_rec.pkid,
                }) {
                    debug!("send pubrec to channel fail, error message:{}, send data time: {}, recv rec time:{}, client_id: {}, pkid: {}, connect_id:{}, diff:{}ms",
                        e,data.create_time, now_millis(), client_id, pub_rec.pkid, connect_id, now_millis() -  data.create_time);
                }
            }
        }

        None
    }

    pub async fn publish_comp(
        &self,
        connect_id: u64,
        pub_comp: &PubComp,
        _: &Option<PubCompProperties>,
    ) -> Option<MqttPacket> {
        if let Some(conn) = self.cache_manager.get_connection(connect_id) {
            let client_id = conn.client_id;
            let pkid = pub_comp.pkid;
            if let Some(data) = self
                .cache_manager
                .pkid_metadata
                .get_ack_packet(&client_id, pkid)
            {
                if let Err(e) = data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubComp,
                    pkid: pub_comp.pkid,
                }) {
                    debug!(
                            "send pubcomp to channel fail, error message:{}, send data time: {}, recv comp time:{}, client_id: {}, pkid: {}, connect_id:{}, diff:{}ms",
                            e,data.create_time, now_millis(), client_id, pub_comp.pkid, connect_id, now_millis() -  data.create_time
                        );
                }
            }
        }
        None
    }

    pub async fn publish_rel(
        &self,
        connect_id: u64,
        pub_rel: &PubRel,
        _: &Option<PubRelProperties>,
    ) -> MqttPacket {
        let connection = if let Some(se) = self.cache_manager.get_connection(connect_id) {
            se
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
                None,
            );
        };

        let client_id = connection.client_id.clone();
        if self
            .cache_manager
            .pkid_metadata
            .get_client_pkid(&client_id, pub_rel.pkid)
            .is_none()
        {
            return response_packet_mqtt_pubcomp_fail(
                &self.protocol,
                &connection,
                pub_rel.pkid,
                PubCompReason::PacketIdentifierNotFound,
                None,
            );
        }

        self.cache_manager
            .pkid_metadata
            .delete_client_pkid(&client_id, pub_rel.pkid);
        connection.recv_qos_message_decr();
        response_packet_mqtt_pubcomp_success(&self.protocol, pub_rel.pkid)
    }

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
        response_packet_mqtt_suback(
            &self.protocol,
            &connection,
            subscribe.packet_identifier,
            return_codes,
            None,
        )
    }

    pub async fn ping(&self, connect_id: u64, _: &PingReq) -> MqttPacket {
        let connection = if let Some(se) = self.cache_manager.get_connection(connect_id) {
            se
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
                None,
            );
        };

        let live_time = ConnectionLiveTime {
            protocol: self.protocol.clone(),
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.cache_manager
            .report_heartbeat(connection.client_id, live_time);
        self.connection_manager
            .report_heartbeat(connect_id, now_second());
        response_packet_mqtt_ping_resp()
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

    pub async fn disconnect(
        &self,
        connect_id: u64,
        disconnect: &Disconnect,
        disconnect_properties: &Option<DisconnectProperties>,
    ) -> Option<MqttPacket> {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return None;
        };

        if let Some(session) = self.cache_manager.get_session_info(&connection.client_id) {
            st_report_disconnected_event(StReportDisconnectedEventContext {
                storage_driver_manager: self.storage_driver_manager.clone(),
                metadata_cache: self.cache_manager.clone(),
                client_pool: self.client_pool.clone(),
                session: session.clone(),
                connection: connection.clone(),
                connect_id,
                connection_manager: self.connection_manager.clone(),
                reason: disconnect.reason_code,
            })
            .await;
        }

        let delete_session = if let Some(properties) = disconnect_properties {
            is_delete_session(&properties.user_properties)
        } else {
            false
        };

        if let Err(e) = disconnect_connection(
            &connection.client_id,
            connect_id,
            &self.cache_manager,
            &self.client_pool,
            &self.connection_manager,
            &self.subscribe_manager,
            delete_session,
        )
        .await
        {
            warn!("disconnect connection failed, {}", e.to_string());
        }

        None
    }
}
