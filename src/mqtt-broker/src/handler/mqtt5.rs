use super::packet::MQTTAckBuild;
use super::packet::{packet_connect_fail, publish_comp_fail, publish_comp_success};
use crate::core::metadata_cache::MetadataCacheManager;
use crate::core::session::get_session_info;
use crate::metadata::connection::{create_connection, get_client_id};
use crate::metadata::topic::{get_topic_info, publish_get_topic_name};
use crate::qos::ack_manager::AckManager;
use crate::qos::QosDataManager;
use crate::subscribe::sub_manager::SubscribeManager;
use crate::subscribe::subscribe::{min_qos, save_retain_message};
use crate::{
    core::client_heartbeat::{ConnectionLiveTime, HeartbeatManager},
    metadata::{message::Message, session::LastWillData},
    qos::memory::QosMemory,
    security::authentication::authentication_login,
    server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use common_base::log::info;
use common_base::{errors::RobustMQError, log::error, tools::now_second};
use protocol::mqtt::{
    Connect, ConnectProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, LastWill, LastWillProperties, Login, MQTTPacket, PingReq, PubAck,
    PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubRec, PubRecProperties, PubRel,
    PubRelProperties, Publish, PublishProperties, QoS, Subscribe, SubscribeProperties,
    SubscribeReasonCode, Unsubscribe, UnsubscribeProperties,
};
use regex::Regex;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct Mqtt5Service<T, S> {
    metadata_cache: Arc<MetadataCacheManager>,
    ack_build: MQTTAckBuild,
    heartbeat_manager: Arc<HeartbeatManager>,
    metadata_storage_adapter: Arc<T>,
    message_storage_adapter: Arc<S>,
    sucscribe_manager: Arc<SubscribeManager>,
    ack_manager: Arc<AckManager>,
}

impl<T, S> Mqtt5Service<T, S>
where
    T: StorageAdapter + Sync + Send + 'static + Clone,
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        metadata_cache: Arc<MetadataCacheManager>,
        ack_build: MQTTAckBuild,
        heartbeat_manager: Arc<HeartbeatManager>,
        metadata_storage_adapter: Arc<T>,
        message_storage_adapter: Arc<S>,
        sucscribe_manager: Arc<SubscribeManager>,
        ack_manager: Arc<AckManager>,
    ) -> Self {
        return Mqtt5Service {
            metadata_cache,
            ack_build,
            heartbeat_manager,
            metadata_storage_adapter,
            message_storage_adapter,
            sucscribe_manager,
            ack_manager,
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
    ) -> MQTTPacket {
        let cluster = self.metadata_cache.get_cluster_info();
        // connect for authentication
        match authentication_login(
            self.metadata_cache.clone(),
            &cluster,
            login,
            &connect_properties,
        )
        .await
        {
            Ok(flag) => {
                if !flag {
                    return packet_connect_fail(ConnectReturnCode::NotAuthorized, None);
                }
            }
            Err(e) => {
                return packet_connect_fail(
                    ConnectReturnCode::ServiceUnavailable,
                    Some(e.to_string()),
                );
            }
        }

        // auto create client id
        let (client_id, new_client_id) = match get_client_id(connnect.client_id.clone()) {
            Ok((client_id, new_client_id)) => (client_id, new_client_id),
            Err(e) => {
                return packet_connect_fail(ConnectReturnCode::BadClientId, Some(e.to_string()));
            }
        };

        // build connection data
        let connection = create_connection(
            connect_id,
            client_id.clone(),
            &cluster,
            connnect.clone(),
            connect_properties.clone(),
        );

        let contain_last_will = !last_will.is_none();
        // save session data
        let (session, new_session) = match get_session_info(
            connect_id,
            client_id.clone(),
            contain_last_will,
            last_will_properties.clone(),
            &cluster,
            &connnect,
            &connect_properties,
            self.metadata_storage_adapter.clone(),
        )
        .await
        {
            Ok(session) => session,
            Err(e) => {
                error(e.to_string());
                return packet_connect_fail(
                    ConnectReturnCode::ServiceUnavailable,
                    Some(e.to_string()),
                );
            }
        };

        // save last will data
        if contain_last_will {
            let last_will = LastWillData {
                last_will,
                last_will_properties,
            };

            let message_storage = MessageStorage::new(self.message_storage_adapter.clone());
            match message_storage
                .save_lastwill(client_id.clone(), last_will)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error(e.to_string());
                    return packet_connect_fail(
                        ConnectReturnCode::ServiceUnavailable,
                        Some(e.to_string()),
                    );
                }
            }
        }

        // update cache
        // When working with locks, the default is to release them as soon as possible
        self.metadata_cache
            .add_session(client_id.clone(), session.clone());
        self.metadata_cache
            .add_connection(connect_id, connection.clone());

        // Record heartbeat information
        let live_time: ConnectionLiveTime = ConnectionLiveTime {
            protobol: crate::server::MQTTProtocol::MQTT5,
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.heartbeat_manager
            .report_hearbeat(connect_id, live_time);

        return self.ack_build.packet_connect_success(
            &cluster,
            client_id.clone(),
            new_client_id,
            session.session_expiry,
            new_session,
        );
    }

    pub async fn publish(
        &self,
        connect_id: u64,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
        idempotent_manager: Arc<QosMemory>,
    ) -> Option<MQTTPacket> {
        let topic_name = match publish_get_topic_name(
            connect_id,
            publish.clone(),
            self.metadata_cache.clone(),
            publish_properties.clone(),
        ) {
            Ok(da) => da,
            Err(e) => {
                return Some(
                    self.ack_build
                        .pub_ack_fail(PubAckReason::UnspecifiedError, Some(e.to_string())),
                );
            }
        };

        let topic = match get_topic_info(
            topic_name,
            self.metadata_cache.clone(),
            self.metadata_storage_adapter.clone(),
            self.message_storage_adapter.clone(),
        )
        .await
        {
            Ok(tp) => tp,
            Err(e) => {
                return Some(
                    self.ack_build
                        .pub_ack_fail(PubAckReason::UnspecifiedError, Some(e.to_string())),
                );
            }
        };

        let connection = if let Some(se) = self.metadata_cache.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return Some(self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            ));
        };

        if publish.payload.len() == 0
            || publish.payload.len() > (connection.max_packet_size as usize)
        {
            return Some(self.ack_build.pub_ack_fail(
                PubAckReason::PayloadFormatInvalid,
                Some(RobustMQError::PacketLenthError(publish.payload.len()).to_string()),
            ));
        };

        let client_id = if let Some(conn) = self.metadata_cache.connection_info.get(&connect_id) {
            conn.client_id.clone()
        } else {
            return Some(self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            ));
        };

        if !idempotent_manager
            .get_qos_pkid_data(client_id.clone(), publish.pkid)
            .await
            .is_none()
        {
            return Some(
                self.ack_build
                    .pub_ack_fail(PubAckReason::PacketIdentifierInUse, None),
            );
        };

        // Persisting retain message data
        let message_storage = MessageStorage::new(self.message_storage_adapter.clone());
        if publish.retain {
            let retain_message = Message::build_message(
                client_id.clone(),
                publish.clone(),
                publish_properties.clone(),
            );
            match message_storage
                .save_retain_message(topic.topic_id.clone(), retain_message)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error(e.to_string());
                    return Some(
                        self.ack_build
                            .distinct(DisconnectReasonCode::UnspecifiedError, Some(e.to_string())),
                    );
                }
            }
        }

        // Persisting stores message data
        let offset = if let Some(record) = Message::build_record(
            client_id.clone(),
            publish.clone(),
            publish_properties.clone(),
        ) {
            match message_storage
                .append_topic_message(topic.topic_id.clone(), vec![record])
                .await
            {
                Ok(da) => {
                    format!("{:?}", da)
                }
                Err(e) => {
                    error(e.to_string());
                    return Some(
                        self.ack_build
                            .distinct(DisconnectReasonCode::UnspecifiedError, Some(e.to_string())),
                    );
                }
            }
        } else {
            "-1".to_string()
        };

        // Pub Ack information is built
        let pkid = publish.pkid;
        let user_properties: Vec<(String, String)> = vec![("offset".to_string(), offset)];
        //ontent is returned according to different QOS levels
        match publish.qos {
            QoS::AtMostOnce => {
                return None;
            }
            QoS::AtLeastOnce => {
                return Some(self.ack_build.pub_ack(pkid, None, user_properties));
            }
            QoS::ExactlyOnce => {
                idempotent_manager
                    .save_qos_pkid_data(connection.client_id, pkid)
                    .await;
                return Some(self.ack_build.pub_rec(pkid, user_properties));
            }
        }
    }

    pub async fn publish_ack(
        &self,
        connect_id: u64,
        pub_ack: PubAck,
        _: Option<PubAckProperties>,
    ) -> Option<MQTTPacket> {
        if let Some(conn) = self.metadata_cache.connection_info.get(&connect_id) {
            let client_id = conn.client_id.clone();
            let pkid = pub_ack.pkid;
            if let Some(data) = self.ack_manager.get(client_id.clone(), pkid) {
                match data.sx.send(true).await {
                    Ok(()) => {
                        self.ack_manager.remove(client_id.clone(), pkid);
                        self.metadata_cache.save_pkid_info(client_id.clone(), pkid);
                    }
                    Err(e) => {
                        error(e.to_string());
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
    ) -> MQTTPacket {
        if let Some(conn) = self.metadata_cache.connection_info.get(&connect_id) {
            let client_id = conn.client_id.clone();
            let pkid = pub_rec.pkid;
            if let Some(data) = self.ack_manager.get(client_id.clone(), pkid) {
                match data.sx.send(true).await {
                    Ok(()) => {}
                    Err(e) => {
                        error(e.to_string());
                    }
                }
            }
        }

        return self.ack_build.pub_rel();
    }

    pub async fn publish_comp(
        &self,
        connect_id: u64,
        pub_comp: PubComp,
        _: Option<PubCompProperties>,
    ) -> MQTTPacket {
        if let Some(conn) = self.metadata_cache.connection_info.get(&connect_id) {
            self.ack_manager
                .remove(conn.client_id.clone(), pub_comp.pkid);
        }
        return self.ack_build.pub_rec(pub_comp.pkid, Vec::new());
    }

    pub async fn publish_rel(
        &self,
        connect_id: u64,
        pub_rel: PubRel,
        _: Option<PubRelProperties>,
        idempotent_manager: Arc<QosMemory>,
    ) -> MQTTPacket {
        let client_id = if let Some(conn) = self.metadata_cache.connection_info.get(&connect_id) {
            conn.client_id.clone()
        } else {
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            );
        };

        if idempotent_manager
            .get_qos_pkid_data(client_id.clone(), pub_rel.pkid)
            .await
            .is_none()
        {
            return publish_comp_fail(pub_rel.pkid);
        };

        idempotent_manager
            .delete_qos_pkid_data(client_id, pub_rel.pkid)
            .await;
        return publish_comp_success(pub_rel.pkid);
    }

    pub async fn subscribe(
        &self,
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
        response_queue_sx: Sender<ResponsePackage>,
        pkid_manager: Arc<QosMemory>,
    ) -> MQTTPacket {
        let client_id = if let Some(conn) = self.metadata_cache.connection_info.get(&connect_id) {
            conn.client_id.clone()
        } else {
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id.clone()).to_string()),
            );
        };

        if !pkid_manager
            .get_sub_pkid_data(client_id.clone(), subscribe.packet_identifier)
            .await
            .is_none()
        {
            return self.ack_build.sub_ack(
                subscribe.packet_identifier,
                vec![SubscribeReasonCode::PkidInUse],
            );
        }

        let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
        let regex = Regex::new(r"^[a-zA-Z0-9#+/]+$").unwrap();
        let cluster_qos = self.metadata_cache.get_cluster_info().max_qos();
        let mut contain_success = false;
        for filter in subscribe.filters.clone() {
            if !regex.is_match(&filter.path) {
                return_codes.push(SubscribeReasonCode::TopicFilterInvalid);
                continue;
            }
            contain_success = true;
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

        if !contain_success {
            return self.ack_build.sub_ack(
                subscribe.packet_identifier,
                vec![SubscribeReasonCode::TopicFilterInvalid],
            );
        }

        pkid_manager
            .save_sub_pkid_data(client_id.clone(), subscribe.packet_identifier)
            .await;

        // Saving subscriptions
        self.metadata_cache.add_client_subscribe(
            client_id.clone(),
            crate::server::MQTTProtocol::MQTT5,
            subscribe.clone(),
            subscribe_properties.clone(),
        );

        self.sucscribe_manager
            .add_subscribe(
                client_id.clone(),
                crate::server::MQTTProtocol::MQTT5,
                subscribe.clone(),
                subscribe_properties.clone(),
            )
            .await;

        // Reservation messages are processed when a subscription is created
        let message_storage = MessageStorage::new(self.message_storage_adapter.clone());
        match save_retain_message(
            connect_id,
            subscribe.clone(),
            subscribe_properties.clone(),
            message_storage,
            self.metadata_cache.clone(),
            response_queue_sx.clone(),
            true,
            false,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                error(e.to_string());
                return self
                    .ack_build
                    .distinct(DisconnectReasonCode::UnspecifiedError, Some(e.to_string()));
            }
        }

        let pkid = subscribe.packet_identifier;
        return self.ack_build.sub_ack(pkid, return_codes);
    }

    pub async fn ping(&self, connect_id: u64, _: PingReq) -> MQTTPacket {
        let connection = if let Some(se) = self.metadata_cache.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            );
        };

        let live_time = ConnectionLiveTime {
            protobol: crate::server::MQTTProtocol::MQTT5,
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.heartbeat_manager
            .report_hearbeat(connect_id, live_time);
        return self.ack_build.ping_resp();
    }

    pub async fn un_subscribe(
        &self,
        connect_id: u64,
        un_subscribe: Unsubscribe,
        _: Option<UnsubscribeProperties>,
        idempotent_manager: Arc<QosMemory>,
    ) -> MQTTPacket {
        let connection = if let Some(se) = self.metadata_cache.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            );
        };

        idempotent_manager
            .delete_sub_pkid_data(connection.client_id.clone(), un_subscribe.pkid)
            .await;

        // Remove subscription information
        if un_subscribe.filters.len() > 0 {
            self.metadata_cache
                .remove_filter(connection.client_id.clone());

            self.sucscribe_manager
                .remove_subscribe(connection.client_id.clone(), un_subscribe.filters);
        }

        return self
            .ack_build
            .unsub_ack(un_subscribe.pkid, None, Vec::new());
    }

    pub async fn disconnect(
        &self,
        connect_id: u64,
        disconnect: Disconnect,
        disconnect_properties: Option<DisconnectProperties>,
    ) -> Option<MQTTPacket> {
        let connection = if let Some(se) = self.metadata_cache.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return None;
        };

        info(format!(
            "Connection [{}] disconnect,disconnect:{:?},disconnect_properties{:?}",
            connect_id, disconnect, disconnect_properties
        ));
        self.metadata_cache
            .remove_connection(connect_id, connection.client_id);
        self.heartbeat_manager.remove_connection(connect_id);
        return None;
    }
}
