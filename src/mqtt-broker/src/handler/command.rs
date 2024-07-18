use super::mqtt::MqttService;
use crate::handler::response::response_packet_mqtt_distinct_by_reason;
use crate::handler::{
    cache_manager::CacheManager, response::response_packet_mqtt_connect_fail,
};
use crate::server::tcp::connection::NetworkConnection;
use crate::server::tcp::connection_manager::ConnectionManager;
use crate::server::tcp::packet::ResponsePackage;
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use clients::poll::ClientPool;
use common_base::log::info;
use protocol::mqtt::common::{
    is_mqtt3, is_mqtt4, is_mqtt5, ConnectReturnCode, DisconnectReasonCode, MQTTPacket, MQTTProtocol,
};
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::{self, Sender};

// S: message storage adapter
#[derive(Clone)]
pub struct Command<S> {
    mqtt3_service: MqttService<S>,
    mqtt4_service: MqttService<S>,
    mqtt5_service: MqttService<S>,
    metadata_cache: Arc<CacheManager>,
    response_queue_sx: Sender<ResponsePackage>,
}

impl<S> Command<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        cache_manager: Arc<CacheManager>,
        message_storage_adapter: Arc<S>,
        response_queue_sx: Sender<ResponsePackage>,
        sucscribe_manager: Arc<SubscribeCacheManager>,
        client_poll: Arc<ClientPool>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        let mqtt3_service = MqttService::new(
            MQTTProtocol::MQTT3,
            cache_manager.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
            stop_sx.clone(),
        );
        let mqtt4_service = MqttService::new(
            MQTTProtocol::MQTT4,
            cache_manager.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
            stop_sx.clone(),
        );
        let mqtt5_service = MqttService::new(
            MQTTProtocol::MQTT5,
            cache_manager.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
            stop_sx.clone(),
        );
        return Command {
            mqtt3_service,
            mqtt4_service,
            mqtt5_service,
            metadata_cache: cache_manager,
            response_queue_sx,
        };
    }

    pub async fn apply(
        &mut self,
        connect_manager: Arc<ConnectionManager>,
        tcp_connection: NetworkConnection,
        addr: SocketAddr,
        packet: MQTTPacket,
    ) -> Option<MQTTPacket> {
        match packet {
            MQTTPacket::Connect(
                protocol_version,
                connect,
                properties,
                last_will,
                last_will_peoperties,
                login,
            ) => {
                connect_manager
                    .set_connect_protocol(tcp_connection.connection_id, protocol_version);

                let resp_pkg = if is_mqtt3(protocol_version) {
                    Some(
                        self.mqtt3_service
                            .connect(
                                tcp_connection.connection_id,
                                connect,
                                properties,
                                last_will,
                                last_will_peoperties,
                                login,
                                addr,
                            )
                            .await,
                    )
                } else if is_mqtt4(protocol_version) {
                    Some(
                        self.mqtt4_service
                            .connect(
                                tcp_connection.connection_id,
                                connect,
                                properties,
                                last_will,
                                last_will_peoperties,
                                login,
                                addr,
                            )
                            .await,
                    )
                } else if is_mqtt5(protocol_version) {
                    Some(
                        self.mqtt5_service
                            .connect(
                                tcp_connection.connection_id,
                                connect,
                                properties,
                                last_will,
                                last_will_peoperties,
                                login,
                                addr,
                            )
                            .await,
                    )
                } else {
                    return Some(response_packet_mqtt_connect_fail(
                        &MQTTProtocol::MQTT4,
                        ConnectReturnCode::UnsupportedProtocolVersion,
                        &None,
                        None,
                    ));
                };

                let ack_pkg = resp_pkg.unwrap();
                if let MQTTPacket::ConnAck(conn_ack, _) = ack_pkg.clone() {
                    if conn_ack.code == ConnectReturnCode::Success {
                        self.metadata_cache
                            .login_success(tcp_connection.connection_id);
                        info(format!(
                            "connect [{}] login success",
                            tcp_connection.connection_id
                        ));
                    }
                }
                return Some(ack_pkg);
            }

            MQTTPacket::Publish(publish, publish_properties) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }

                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish(tcp_connection.connection_id, publish, publish_properties)
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish(tcp_connection.connection_id, publish, publish_properties)
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish(tcp_connection.connection_id, publish, publish_properties)
                        .await;
                }
            }

            MQTTPacket::PubRec(pub_rec, pub_rec_properties) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }
                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                        .await;
                }
                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                        .await;
                }
                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                        .await;
                }
            }

            MQTTPacket::PubComp(pub_comp, pub_comp_properties) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }

                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                        .await;
                }
            }

            MQTTPacket::PubRel(pub_rel, pub_rel_properties) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }
                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                            .await,
                    );
                }
                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                            .await,
                    );
                }
            }

            MQTTPacket::PubAck(pub_ack, pub_ack_properties) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }

                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                        .await;
                }
                return None;
            }

            MQTTPacket::Subscribe(subscribe, subscribe_properties) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }
                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .subscribe(
                                tcp_connection.connection_id,
                                subscribe,
                                subscribe_properties,
                                self.response_queue_sx.clone(),
                            )
                            .await,
                    );
                }
                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .subscribe(
                                tcp_connection.connection_id,
                                subscribe,
                                subscribe_properties,
                                self.response_queue_sx.clone(),
                            )
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .subscribe(
                                tcp_connection.connection_id,
                                subscribe,
                                subscribe_properties,
                                self.response_queue_sx.clone(),
                            )
                            .await,
                    );
                }
            }

            MQTTPacket::PingReq(ping) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }

                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .ping(tcp_connection.connection_id, ping)
                            .await,
                    );
                }

                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .ping(tcp_connection.connection_id, ping)
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .ping(tcp_connection.connection_id, ping)
                            .await,
                    );
                }
            }

            MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                if !self.auth_login(tcp_connection.connection_id).await {
                    return Some(self.un_login_err(tcp_connection.connection_id));
                }

                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .un_subscribe(
                                tcp_connection.connection_id,
                                unsubscribe,
                                unsubscribe_properties,
                            )
                            .await,
                    );
                }

                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .un_subscribe(
                                tcp_connection.connection_id,
                                unsubscribe,
                                unsubscribe_properties,
                            )
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .un_subscribe(
                                tcp_connection.connection_id,
                                unsubscribe,
                                unsubscribe_properties,
                            )
                            .await,
                    );
                }
            }

            MQTTPacket::Disconnect(disconnect, disconnect_properties) => {
                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .disconnect(
                            tcp_connection.connection_id,
                            disconnect,
                            disconnect_properties,
                        )
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .disconnect(
                            tcp_connection.connection_id,
                            disconnect,
                            disconnect_properties,
                        )
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .disconnect(
                            tcp_connection.connection_id,
                            disconnect,
                            disconnect_properties,
                        )
                        .await;
                }
            }

            _ => {
                return Some(response_packet_mqtt_connect_fail(
                    &MQTTProtocol::MQTT5,
                    ConnectReturnCode::MalformedPacket,
                    &None,
                    None,
                ));
            }
        }
        return Some(response_packet_mqtt_connect_fail(
            &MQTTProtocol::MQTT5,
            ConnectReturnCode::UnsupportedProtocolVersion,
            &None,
            None,
        ));
    }

    fn un_login_err(&self, connection_id: u64) -> MQTTPacket {
        info(format!("connect id [{}] Not logged in", connection_id));
        return response_packet_mqtt_distinct_by_reason(
            &MQTTProtocol::MQTT5,
            Some(DisconnectReasonCode::NotAuthorized),
        );
    }

    pub async fn auth_login(&self, connection_id: u64) -> bool {
        return self.metadata_cache.is_login(connection_id);
    }
}
