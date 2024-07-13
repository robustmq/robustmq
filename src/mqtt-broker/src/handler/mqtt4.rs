use crate::core::cache_manager::ConnectionLiveTime;
use crate::core::connection::{build_connection, get_client_id};
use crate::core::lastwill::save_last_will_message;
use crate::core::response_packet::{
    response_packet_matt5_puback_fail, response_packet_matt_connect_fail,
    response_packet_matt_connect_success, response_packet_matt_distinct,
    response_packet_matt_distinct_by_reason,
};
use crate::core::session::{build_session, save_session};
use crate::core::topic::get_topic_name;
use crate::core::validator::publish_validator;
use crate::core::{cache_manager::CacheManager, validator::connect_validator};
use crate::security::authentication::authentication_login;
use crate::storage::session::SessionStorage;
use clients::poll::ClientPool;
use common_base::tools::now_second;
use protocol::mqtt::common::{
    Connect, ConnectReturnCode, Disconnect, DisconnectReasonCode, LastWill, Login, MQTTPacket,
    MQTTProtocol, PingReq, PubAck, PubAckReason, PubRecReason, Publish, QoS, Subscribe,
    Unsubscribe,
};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct Mqtt4Service {
    client_poll: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
}

impl Mqtt4Service {
    pub fn new(client_poll: Arc<ClientPool>, cache_manager: Arc<CacheManager>) -> Self {
        return Mqtt4Service {
            client_poll,
            cache_manager,
        };
    }

    pub async fn connect(
        &mut self,
        connect_id: u64,
        connnect: Connect,
        last_will: Option<LastWill>,
        login: Option<Login>,
        addr: SocketAddr,
    ) -> MQTTPacket {
        let cluster = self.cache_manager.get_cluster_info();
        if let Some(res) = connect_validator(
            &MQTTProtocol::MQTT4,
            &cluster,
            &connnect,
            &None,
            &last_will,
            &None,
            &login,
            &addr,
        ) {
            return res;
        }

        match authentication_login(&self.cache_manager, &login, &None, &addr).await {
            Ok(flag) => {
                if !flag {
                    return response_packet_matt_connect_fail(
                        &MQTTProtocol::MQTT4,
                        ConnectReturnCode::NotAuthorized,
                        &None,
                        None,
                    );
                }
            }
            Err(e) => {
                return response_packet_matt_connect_fail(
                    &MQTTProtocol::MQTT4,
                    ConnectReturnCode::UnspecifiedError,
                    &None,
                    Some(e.to_string()),
                );
            }
        }
        let (client_id, new_client_id) = get_client_id(&connnect.client_id);

        let connection = build_connection(connect_id, &client_id, &cluster, &connnect, &None);

        let (session, new_session) = match build_session(
            connect_id,
            &client_id,
            &connnect,
            &None,
            &last_will,
            &None,
            &self.client_poll,
            &self.cache_manager,
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                return response_packet_matt_connect_fail(
                    &MQTTProtocol::MQTT5,
                    ConnectReturnCode::MalformedPacket,
                    &None,
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
                return response_packet_matt_connect_fail(
                    &MQTTProtocol::MQTT5,
                    ConnectReturnCode::MalformedPacket,
                    &None,
                    Some(e.to_string()),
                );
            }
        }

        match save_last_will_message(&client_id, &last_will, &None, &self.client_poll).await {
            Ok(()) => {}
            Err(e) => {
                return response_packet_matt_connect_fail(
                    &MQTTProtocol::MQTT5,
                    ConnectReturnCode::UnspecifiedError,
                    &None,
                    Some(e.to_string()),
                );
            }
        }
        let live_time = ConnectionLiveTime {
            protobol: MQTTProtocol::MQTT5,
            keep_live: connection.keep_alive as u16,
            heartbeat: now_second(),
        };
        self.cache_manager.report_heartbeat(&client_id, live_time);

        self.cache_manager
            .add_session(client_id.clone(), session.clone());
        self.cache_manager
            .add_connection(connect_id, connection.clone());

        return response_packet_matt_connect_success(
            MQTTProtocol::MQTT5,
            &cluster,
            client_id.clone(),
            new_client_id,
            session.session_expiry as u32,
            new_session,
            &None,
        );
    }

    pub async fn disconnect(&self, connect_id: u64, _: Disconnect) -> Option<MQTTPacket> {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return None;
        };

        self.cache_manager.remove_connection(connect_id);
        self.cache_manager
            .update_session_connect_id(&connection.client_id, None);

        let session_storage = SessionStorage::new(self.client_poll.clone());
        match session_storage
            .update_session(&connection.client_id, 0, 0, 0, now_second())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Some(response_packet_matt_distinct(
                    MQTTProtocol::MQTT4,
                    None,
                    &connection,
                    Some(e.to_string()),
                ));
            }
        }

        return Some(response_packet_matt_distinct(
            MQTTProtocol::MQTT4,
            None,
            &connection,
            None,
        ));
    }

    pub async fn publish(&self, connect_id: u64, publish: Publish) -> Option<MQTTPacket> {
        let connection = if let Some(se) = self.cache_manager.connection_info.get(&connect_id) {
            se.clone()
        } else {
            return Some(response_packet_matt_distinct_by_reason(
                MQTTProtocol::MQTT5,
                Some(DisconnectReasonCode::MaximumConnectTime),
            ));
        };

        if let Some(pkg) = publish_validator(
            &self.cache_manager,
            &self.client_poll,
            &connection,
            &publish,
            &None,
        )
        .await
        {
            if publish.qos == QoS::AtMostOnce {
                return None;
            } else {
                return Some(pkg);
            }
        }

        let is_puback = publish.qos != QoS::ExactlyOnce;

        let topic_name = match get_topic_name(connect_id, &self.cache_manager, &publish, &None) {
            Ok(da) => da,
            Err(e) => {
                if is_puback {
                    return Some(response_packet_matt5_puback_fail(
                        &connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                } else {
                    return Some(response_packet_matt5_pubrec_fail(
                        &connection,
                        publish.pkid,
                        PubRecReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                }
            }
        };

        let topic = match try_init_topic(
            topic_name.clone(),
            self.cache_manager.clone(),
            self.message_storage_adapter.clone(),
            self.client_poll.clone(),
        )
        .await
        {
            Ok(tp) => tp,
            Err(e) => {
                if is_puback {
                    return Some(response_packet_matt5_puback_fail(
                        &connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                } else {
                    return Some(response_packet_matt5_pubrec_fail(
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
                if is_puback {
                    return Some(response_packet_matt5_puback_fail(
                        &connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(e.to_string()),
                    ));
                } else {
                    return Some(response_packet_matt5_pubrec_fail(
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
                    if is_puback {
                        return Some(response_packet_matt5_puback_fail(
                            &connection,
                            publish.pkid,
                            PubAckReason::UnspecifiedError,
                            Some(e.to_string()),
                        ));
                    } else {
                        return Some(response_packet_matt5_pubrec_fail(
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
                let reason_code = if path_contain_sub(&topic_name) {
                    PubAckReason::Success
                } else {
                    PubAckReason::NoMatchingSubscribers
                };
                return Some(response_packet_matt5_puback_success(
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
                        if is_puback {
                            return Some(response_packet_matt5_puback_fail(
                                &connection,
                                publish.pkid,
                                PubAckReason::UnspecifiedError,
                                Some(e.to_string()),
                            ));
                        } else {
                            return Some(response_packet_matt5_pubrec_fail(
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

                return Some(response_packet_matt5_pubrec_success(
                    reason_code,
                    publish.pkid,
                    user_properties,
                ));
            }
        }
    }

    pub fn publish_ack(&self, pub_ack: PubAck) {}

    pub fn subscribe(&self, subscribe: Subscribe) -> Option<MQTTPacket> {
        return None;
    }

    pub fn ping(&self, ping: PingReq) -> Option<MQTTPacket> {
        return None;
    }

    pub fn un_subscribe(&self, un_subscribe: Unsubscribe) -> Option<MQTTPacket> {
        return None;
    }

    fn un_login_err(&self) -> Option<MQTTPacket> {
        return None;
    }
}
