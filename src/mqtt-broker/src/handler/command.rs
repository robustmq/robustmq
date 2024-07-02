use super::mqtt3::Mqtt3Service;
use super::mqtt4::Mqtt4Service;
use super::mqtt5::Mqtt5Service;
use super::packet::MQTTAckBuild;
use crate::core::{cache_manager::CacheManager, connection::response_packet_matt5_connect_fail};
use crate::server::tcp::connection::TCPConnection;
use crate::server::tcp::connection_manager::ConnectionManager;
use crate::server::tcp::packet::ResponsePackage;
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use clients::poll::ClientPool;
use common_base::log::info;
use protocol::mqtt::common::{ConnectReturnCode, MQTTPacket};
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::{self, Sender};

// S: message storage adapter
#[derive(Clone)]
pub struct Command<S> {
    ack_build: MQTTAckBuild,
    mqtt3_service: Mqtt3Service,
    mqtt4_service: Mqtt4Service,
    mqtt5_service: Mqtt5Service<S>,
    metadata_cache: Arc<CacheManager>,
    response_queue_sx: Sender<ResponsePackage>,
    client_poll: Arc<ClientPool>,
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
        let ack_build = MQTTAckBuild::new(cache_manager.clone());
        let mqtt4_service = Mqtt4Service::new(cache_manager.clone(), ack_build.clone());
        let mqtt5_service = Mqtt5Service::new(
            cache_manager.clone(),
            ack_build.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
            stop_sx.clone(),
        );
        let mqtt3_service = Mqtt3Service::new(cache_manager.clone(), ack_build.clone());
        return Command {
            ack_build,
            mqtt3_service,
            mqtt4_service,
            mqtt5_service,
            metadata_cache: cache_manager,
            response_queue_sx,
            client_poll,
        };
    }

    pub async fn apply(
        &mut self,
        connect_manager: Arc<ConnectionManager>,
        tcp_connection: TCPConnection,
        addr: SocketAddr,
        packet: MQTTPacket,
    ) -> Option<MQTTPacket> {
        info(format!("revc packet:{:?}", packet));
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

                let ack_pkg = if protocol_version == 3 {
                    self.mqtt3_service
                        .connect(connect.clone(), last_will.clone(), login.clone())
                        .await
                } else if protocol_version == 4 {
                    self.mqtt4_service
                        .connect(connect.clone(), last_will.clone(), login.clone())
                        .await
                } else if protocol_version == 5 {
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
                        .await
                } else {
                    return Some(response_packet_matt5_connect_fail(
                        ConnectReturnCode::UnsupportedProtocolVersion,
                        &properties,
                        None,
                    ));
                };

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
                    return Some(self.mqtt3_service.publish(publish));
                }

                if tcp_connection.is_mqtt4() {
                    return Some(self.mqtt4_service.publish(publish));
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
                    return None;
                }
                if tcp_connection.is_mqtt4() {
                    return None;
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
                    return None;
                }

                if tcp_connection.is_mqtt4() {
                    return None;
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
                    return None;
                }
                if tcp_connection.is_mqtt4() {
                    return None;
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
                    return None;
                }
                if tcp_connection.is_mqtt4() {
                    self.mqtt4_service.publish_ack(pub_ack.clone());
                }

                if tcp_connection.is_mqtt5() {
                    self.mqtt5_service
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
                    return None;
                }
                if tcp_connection.is_mqtt4() {
                    return Some(self.mqtt4_service.subscribe(subscribe));
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
                    return None;
                }

                if tcp_connection.is_mqtt4() {
                    return Some(self.mqtt4_service.ping(ping));
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
                    return None;
                }

                if tcp_connection.is_mqtt4() {
                    return Some(self.mqtt4_service.un_subscribe(unsubscribe));
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
                    return Some(self.mqtt3_service.disconnect(disconnect));
                }

                if tcp_connection.is_mqtt4() {
                    return Some(self.mqtt4_service.disconnect(disconnect));
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
                return Some(self.ack_build.distinct(
                    protocol::mqtt::common::DisconnectReasonCode::MalformedPacket,
                    None,
                ));
            }
        }
        return Some(self.ack_build.distinct(
            protocol::mqtt::common::DisconnectReasonCode::ProtocolError,
            None,
        ));
    }

    fn un_login_err(&self, connection_id: u64) -> MQTTPacket {
        info(format!("connect id [{}] Not logged in", connection_id));
        return self.ack_build.distinct(
            protocol::mqtt::common::DisconnectReasonCode::NotAuthorized,
            None,
        );
    }

    pub async fn auth_login(&self, connection_id: u64) -> bool {
        return self.metadata_cache.is_login(connection_id);
    }
}
