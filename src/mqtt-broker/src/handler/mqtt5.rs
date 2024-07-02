use super::packet::MQTTAckBuild;
use super::packet::{publish_comp_fail, publish_comp_success};
use crate::core::cache_manager::{CacheManager, ConnectionLiveTime};
use crate::core::cache_manager::{QosAckPackageData, QosAckPackageType};
use crate::core::connection::{create_connection, get_client_id};
use crate::core::lastwill::save_last_will_message;
use crate::core::response_packet::{
    response_packet_matt5_connect_fail, response_packet_matt5_connect_success,
};
use crate::core::retain::{save_topic_retain_message, send_retain_message};
use crate::core::session::save_session;
use crate::core::topic::{get_topic_info, get_topic_name, save_topic_alias};
use crate::core::validator::connect_params_validator;
use crate::storage::session::SessionStorage;
use crate::subscribe::sub_common::{min_qos, sub_path_validator};
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use crate::{
    security::authentication::authentication_login, server::tcp::packet::ResponsePackage,
    storage::message::MessageStorage,
};
use clients::poll::ClientPool;
use common_base::{errors::RobustMQError, log::error, tools::now_second};
use metadata_struct::mqtt::message::MQTTMessage;
use protocol::mqtt::common::{
    Connect, ConnectProperties, ConnectReturnCode, Disconnect, DisconnectProperties,
    DisconnectReasonCode, LastWill, LastWillProperties, Login, MQTTPacket, MQTTProtocol, PingReq,
    PubAck, PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubRec, PubRecProperties,
    PubRel, PubRelProperties, PubRelReason, Publish, PublishProperties, QoS, Subscribe,
    SubscribeProperties, SubscribeReasonCode, Unsubscribe, UnsubscribeProperties,
};
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::{self, Sender};

#[derive(Clone)]
pub struct Mqtt5Service<S> {
    cache_manager: Arc<CacheManager>,
    ack_build: MQTTAckBuild,
    message_storage_adapter: Arc<S>,
    sucscribe_cache: Arc<SubscribeCacheManager>,
    client_poll: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
}

impl<S> Mqtt5Service<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        cache_manager: Arc<CacheManager>,
        ack_build: MQTTAckBuild,
        message_storage_adapter: Arc<S>,
        sucscribe_manager: Arc<SubscribeCacheManager>,
        client_poll: Arc<ClientPool>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        return Mqtt5Service {
            cache_manager,
            ack_build,
            message_storage_adapter,
            sucscribe_cache: sucscribe_manager,
            client_poll,
            stop_sx,
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

        if let Some(res) = connect_params_validator(
            &cluster,
            &connnect,
            &connect_properties,
            &last_will,
            &last_will_properties,
            &login,
        ) {
            return res;
        }

        match authentication_login(self.cache_manager.clone(), login, &connect_properties, addr)
            .await
        {
            Ok(flag) => {
                if !flag {
                    return response_packet_matt5_connect_fail(
                        ConnectReturnCode::NotAuthorized,
                        &connect_properties,
                        None,
                    );
                }
            }
            Err(e) => {
                return response_packet_matt5_connect_fail(
                    ConnectReturnCode::UnspecifiedError,
                    &connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        let (client_id, new_client_id) = get_client_id(connnect.client_id.clone());

        let (session, new_session) = match save_session(
            connect_id,
            client_id.clone(),
            connnect.clone(),
            connect_properties.clone(),
            last_will.clone(),
            last_will_properties.clone(),
            self.client_poll.clone(),
            self.cache_manager.clone(),
        )
        .await
        {
            Ok(session) => session,
            Err(e) => {
                return response_packet_matt5_connect_fail(
                    ConnectReturnCode::MalformedPacket,
                    &connect_properties,
                    Some(e.to_string()),
                );
            }
        };

        match save_last_will_message(
            client_id.clone(),
            last_will.clone(),
            last_will_properties.clone(),
            self.client_poll.clone(),
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                error(e.to_string());
                return response_packet_matt5_connect_fail(
                    ConnectReturnCode::UnspecifiedError,
                    &connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        let connection = create_connection(
            connect_id,
            client_id.clone(),
            &cluster,
            connnect.clone(),
            connect_properties.clone(),
        );

        let live_time: ConnectionLiveTime = ConnectionLiveTime {
            protobol: MQTTProtocol::MQTT5,
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.cache_manager
            .report_heartbeat(client_id.clone(), live_time);
        self.cache_manager
            .add_session(client_id.clone(), session.clone());
        self.cache_manager
            .add_connection(connect_id, connection.clone());

        return response_packet_matt5_connect_success(
            &cluster,
            client_id.clone(),
            new_client_id,
            session.session_expiry as u32,
            new_session,
            &connect_properties,
        );
    }

    pub async fn publish(
        &self,
        connect_id: u64,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> Option<MQTTPacket> {
        match save_topic_alias(
            connect_id,
            publish.topic.clone(),
            self.cache_manager.clone(),
            publish_properties.clone(),
        ) {
            Ok(()) => {}
            Err(e) => {
                return Some(
                    self.ack_build
                        .pub_ack_fail(PubAckReason::TopicNameInvalid, Some(e.to_string())),
                );
            }
        }

        let topic_name = match get_topic_name(
            connect_id,
            publish.clone(),
            self.cache_manager.clone(),
            publish_properties.clone(),
        ) {
            Ok(da) => da,
            Err(e) => {
                return Some(
                    self.ack_build
                        .pub_ack_fail(PubAckReason::TopicNameInvalid, Some(e.to_string())),
                );
            }
        };

        let topic = match get_topic_info(
            topic_name.clone(),
            self.cache_manager.clone(),
            self.message_storage_adapter.clone(),
            self.client_poll.clone(),
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

        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return Some(self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            ));
        };

        let client_id = if let Some(conn) = self.cache_manager.connection_info.get(&connect_id) {
            conn.client_id.clone()
        } else {
            return Some(self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            ));
        };

        if publish.qos == QoS::ExactlyOnce {
            if !self
                .cache_manager
                .get_client_pkid(client_id.clone(), publish.pkid)
                .await
                .is_none()
            {
                return Some(
                    self.ack_build
                        .pub_ack_fail(PubAckReason::PacketIdentifierInUse, None),
                );
            };
        }

        // Persisting retain message data
        match save_topic_retain_message(
            topic_name.clone(),
            client_id.clone(),
            publish.clone(),
            self.cache_manager.clone(),
            publish_properties.clone(),
            self.client_poll.clone(),
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                return Some(
                    self.ack_build
                        .pub_ack_fail(PubAckReason::UnspecifiedError, Some(e.to_string())),
                );
            }
        }

        // Persisting stores message data
        let message_storage = MessageStorage::new(self.message_storage_adapter.clone());
        let offset = if let Some(record) = MQTTMessage::build_record(
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
                self.cache_manager
                    .add_client_pkid(connection.client_id, pkid)
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
                        error(format!(
                            "publish ack send ack manager message error, error message:{}",
                            e.to_string()
                        ));
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
                        error(format!(
                            "publish rec send ack manager message error, error message:{}",
                            e.to_string()
                        ));
                    }
                }
            }
        }

        return Some(self.ack_build.pub_rel(pub_rec.pkid, PubRelReason::Success));
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
                        error(format!(
                            "publish comp send ack manager message error, error message:{}",
                            e.to_string()
                        ));
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
        let client_id = if let Some(conn) = self.cache_manager.connection_info.get(&connect_id) {
            conn.client_id.clone()
        } else {
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            );
        };

        if self
            .cache_manager
            .get_client_pkid(client_id.clone(), pub_rel.pkid)
            .await
            .is_none()
        {
            return publish_comp_fail(pub_rel.pkid);
        };

        self.cache_manager
            .delete_client_pkid(client_id, pub_rel.pkid)
            .await;
        return publish_comp_success(pub_rel.pkid);
    }

    pub async fn subscribe(
        &self,
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
        response_queue_sx: Sender<ResponsePackage>,
    ) -> MQTTPacket {
        let client_id = if let Some(conn) = self.cache_manager.connection_info.get(&connect_id) {
            conn.client_id.clone()
        } else {
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id.clone()).to_string()),
            );
        };

        if !self
            .cache_manager
            .get_client_pkid(client_id.clone(), subscribe.packet_identifier)
            .await
            .is_none()
        {
            return self.ack_build.sub_ack(
                subscribe.packet_identifier,
                vec![SubscribeReasonCode::PkidInUse],
                None,
            );
        }

        let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
        let cluster_qos = self.cache_manager.get_cluster_info().max_qos();
        let mut contain_success = false;
        for filter in subscribe.filters.clone() {
            if !sub_path_validator(filter.path) {
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
                None,
            );
        }

        self.cache_manager
            .add_client_pkid(client_id.clone(), subscribe.packet_identifier)
            .await;

        // Saving subscriptions
        self.cache_manager.add_client_subscribe(
            client_id.clone(),
            MQTTProtocol::MQTT5,
            subscribe.clone(),
            subscribe_properties.clone(),
        );

        self.sucscribe_cache
            .add_subscribe(
                client_id.clone(),
                MQTTProtocol::MQTT5,
                subscribe.clone(),
                subscribe_properties.clone(),
            )
            .await;

        send_retain_message(
            client_id.clone(),
            subscribe.clone(),
            subscribe_properties.clone(),
            self.client_poll.clone(),
            self.cache_manager.clone(),
            response_queue_sx.clone(),
            self.stop_sx.clone(),
        )
        .await;

        let pkid = subscribe.packet_identifier;
        return self.ack_build.sub_ack(pkid, return_codes, None);
    }

    pub async fn ping(&self, connect_id: u64, _: PingReq) -> MQTTPacket {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            );
        };

        let live_time = ConnectionLiveTime {
            protobol: MQTTProtocol::MQTT5,
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.cache_manager
            .report_heartbeat(connection.client_id.clone(), live_time);
        return self.ack_build.ping_resp();
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
            return self.ack_build.distinct(
                DisconnectReasonCode::UnspecifiedError,
                Some(RobustMQError::NotFoundConnectionInCache(connect_id).to_string()),
            );
        };

        self.cache_manager
            .delete_client_pkid(connection.client_id.clone(), un_subscribe.pkid)
            .await;

        self.sucscribe_cache
            .remove_subscribe(connection.client_id.clone(), un_subscribe.filters.clone());

        self.cache_manager
            .remove_filter_by_pkid(connection.client_id.clone(), un_subscribe.filters);

        return self
            .ack_build
            .unsub_ack(un_subscribe.pkid, None, Vec::new());
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

        self.cache_manager.remove_connection(connect_id);

        let session_storage = SessionStorage::new(self.client_poll.clone());
        match session_storage
            .update_session(connection.client_id, 0, 0, 0, now_second())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error(e.to_string());
            }
        }

        return Some(
            self.ack_build
                .distinct(DisconnectReasonCode::NormalDisconnection, None),
        );
    }
}
