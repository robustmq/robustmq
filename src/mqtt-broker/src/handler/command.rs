use super::mqtt4::Mqtt4Service;
use super::mqtt5::Mqtt5Service;
use super::packet::{packet_connect_fail, MQTTAckBuild};
use crate::core::heartbeat_cache::HeartbeatCache;
use crate::core::metadata_cache::MetadataCacheManager;
use crate::core::qos_manager::QosManager;
use crate::server::tcp::packet::ResponsePackage;
use crate::server::MQTTProtocol;
use crate::subscribe::subscribe_cache::SubscribeCache;
use clients::poll::ClientPool;
use common_base::log::info;
use protocol::mqtt::{ConnectReturnCode, MQTTPacket};
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::Sender;

// S: message storage adapter
#[derive(Clone)]
pub struct Command<S> {
    protocol: MQTTProtocol,
    ack_build: MQTTAckBuild,
    mqtt4_service: Mqtt4Service,
    mqtt5_service: Mqtt5Service<S>,
    metadata_cache: Arc<MetadataCacheManager>,
    response_queue_sx: Sender<ResponsePackage>,
    qos_manager: Arc<QosManager>,
    client_poll: Arc<ClientPool>,
}

impl<S> Command<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        protocol: MQTTProtocol,
        metadata_cache: Arc<MetadataCacheManager>,
        heartbeat_manager: Arc<HeartbeatCache>,
        message_storage_adapter: Arc<S>,
        response_queue_sx: Sender<ResponsePackage>,
        qos_manager: Arc<QosManager>,
        sucscribe_manager: Arc<SubscribeCache>,
        ack_manager: Arc<QosManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        let ack_build = MQTTAckBuild::new(protocol.clone(), metadata_cache.clone());
        let mqtt4_service = Mqtt4Service::new(
            metadata_cache.clone(),
            ack_build.clone(),
            heartbeat_manager.clone(),
        );
        let mqtt5_service = Mqtt5Service::new(
            metadata_cache.clone(),
            ack_build.clone(),
            heartbeat_manager.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            ack_manager.clone(),
            client_poll.clone(),
        );
        return Command {
            protocol,
            ack_build,
            mqtt4_service,
            mqtt5_service,
            metadata_cache,
            response_queue_sx,
            qos_manager,
            client_poll,
        };
    }

    pub async fn apply(
        &mut self,
        connect_id: u64,
        addr: SocketAddr,
        packet: MQTTPacket,
    ) -> Option<MQTTPacket> {
        info(format!("revc packet:{:?}", packet));
        match packet {
            MQTTPacket::Connect(connect, properties, last_will, last_will_peoperties, login) => {
                if self.protocol != MQTTProtocol::MQTT4 && self.protocol != MQTTProtocol::MQTT5 {}

                let ack_pkg = if self.protocol == MQTTProtocol::MQTT4 {
                    self.mqtt4_service
                        .connect(connect.clone(), last_will.clone(), login.clone())
                        .await
                } else if self.protocol == MQTTProtocol::MQTT5 {
                    self.mqtt5_service
                        .connect(
                            connect_id,
                            connect,
                            properties,
                            last_will,
                            last_will_peoperties,
                            login,
                            addr,
                        )
                        .await
                } else {
                    return Some(packet_connect_fail(
                        ConnectReturnCode::UnsupportedProtocolVersion,
                        None,
                    ));
                };

                if let MQTTPacket::ConnAck(conn_ack, _) = ack_pkg.clone() {
                    if conn_ack.code == ConnectReturnCode::Success {
                        self.metadata_cache.login_success(connect_id);
                        info(format!("connect [{}] login success", connect_id));
                    }
                }
                return Some(ack_pkg);
            }

            MQTTPacket::Publish(publish, publish_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }

                if self.protocol == MQTTProtocol::MQTT4 {
                    return Some(self.mqtt4_service.publish(publish));
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self
                        .mqtt5_service
                        .publish(
                            connect_id,
                            publish,
                            publish_properties,
                            self.qos_manager.clone(),
                        )
                        .await;
                }
            }

            MQTTPacket::PubRec(pub_rec, pub_rec_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return None;
                }
                if self.protocol == MQTTProtocol::MQTT5 {
                    return self
                        .mqtt5_service
                        .publish_rec(connect_id, pub_rec, pub_rec_properties)
                        .await;
                }
            }

            MQTTPacket::PubComp(pub_comp, pub_comp_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return None;
                }
                if self.protocol == MQTTProtocol::MQTT5 {
                    return self
                        .mqtt5_service
                        .publish_comp(connect_id, pub_comp, pub_comp_properties)
                        .await;
                }
            }

            MQTTPacket::PubRel(pub_rel, pub_rel_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return None;
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return Some(
                        self.mqtt5_service
                            .publish_rel(
                                connect_id,
                                pub_rel,
                                pub_rel_properties,
                                self.qos_manager.clone(),
                            )
                            .await,
                    );
                }
            }

            MQTTPacket::PubAck(pub_ack, pub_ack_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    self.mqtt4_service.publish_ack(pub_ack.clone());
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    self.mqtt5_service
                        .publish_ack(connect_id, pub_ack, pub_ack_properties)
                        .await;
                }
                return None;
            }

            MQTTPacket::Subscribe(subscribe, subscribe_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return Some(self.mqtt4_service.subscribe(subscribe));
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return Some(
                        self.mqtt5_service
                            .subscribe(
                                connect_id,
                                subscribe,
                                subscribe_properties,
                                self.response_queue_sx.clone(),
                                self.qos_manager.clone(),
                            )
                            .await,
                    );
                }
            }

            MQTTPacket::PingReq(ping) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }

                if self.protocol == MQTTProtocol::MQTT4 {
                    return Some(self.mqtt4_service.ping(ping));
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return Some(self.mqtt5_service.ping(connect_id, ping).await);
                }
            }

            MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return Some(self.mqtt4_service.un_subscribe(unsubscribe));
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return Some(
                        self.mqtt5_service
                            .un_subscribe(
                                connect_id,
                                unsubscribe,
                                unsubscribe_properties,
                                self.qos_manager.clone(),
                            )
                            .await,
                    );
                }
            }

            MQTTPacket::Disconnect(disconnect, disconnect_properties) => {
                if !self.auth_login(connect_id).await {
                    return Some(self.un_login_err(connect_id));
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return Some(self.mqtt4_service.disconnect(disconnect));
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self
                        .mqtt5_service
                        .disconnect(connect_id, disconnect, disconnect_properties)
                        .await;
                }
            }

            _ => {
                return Some(self.ack_build.distinct(
                    protocol::mqtt::DisconnectReasonCode::ImplementationSpecificError,
                    None,
                ));
            }
        }
        return Some(self.ack_build.distinct(
            protocol::mqtt::DisconnectReasonCode::ImplementationSpecificError,
            None,
        ));
    }

    fn un_login_err(&self, connect_id: u64) -> MQTTPacket {
        info(format!("connect id [{}] Not logged in", connect_id));
        return self
            .ack_build
            .distinct(protocol::mqtt::DisconnectReasonCode::NotAuthorized, None);
    }

    pub async fn auth_login(&self, connect_id: u64) -> bool {
        return self.metadata_cache.is_login(connect_id);
    }
}
