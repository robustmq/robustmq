use super::mqtt4::Mqtt4Service;
use super::mqtt5::Mqtt5Service;
use super::packet::{packet_connect_fail, MQTTAckBuild};
use crate::core::heartbeat_manager::HeartbeatManager;
use crate::idempotent::memory::IdempotentMemory;
use crate::server::tcp::packet::ResponsePackage;
use crate::subscribe::manager::SubScribeManager;
use crate::{metadata::cache::MetadataCacheManager, server::MQTTProtocol};
use common_base::log::info;
use protocol::mqtt::{ConnectReturnCode, MQTTPacket};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::Sender;

// T: metadata storage adapter
// S: message storage adapter
#[derive(Clone)]
pub struct Command<T, S> {
    protocol: MQTTProtocol,
    ack_build: MQTTAckBuild<T>,
    mqtt4_service: Mqtt4Service<T>,
    mqtt5_service: Mqtt5Service<T, S>,
    metadata_cache: Arc<MetadataCacheManager<T>>,
    response_queue_sx: Sender<ResponsePackage>,
    idempotent_manager: Arc<IdempotentMemory>,
}

impl<T, S> Command<T, S>
where
    T: StorageAdapter + Sync + Send + 'static + Clone,
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        protocol: MQTTProtocol,
        metadata_cache: Arc<MetadataCacheManager<T>>,
        heartbeat_manager: Arc<HeartbeatManager>,
        subscribe_manager: Arc<SubScribeManager<T>>,
        metadata_storage_adapter: Arc<T>,
        message_storage_adapter: Arc<S>,
        response_queue_sx: Sender<ResponsePackage>,
        idempotent_manager: Arc<IdempotentMemory>,
    ) -> Self {
        let ack_build = MQTTAckBuild::new(protocol.clone(), metadata_cache.clone());
        let mqtt4_service = Mqtt4Service::new(
            metadata_cache.clone(),
            ack_build.clone(),
            heartbeat_manager.clone(),
        );
        let mqtt5_service = Mqtt5Service::new(
            metadata_cache.clone(),
            subscribe_manager,
            ack_build.clone(),
            heartbeat_manager.clone(),
            metadata_storage_adapter.clone(),
            message_storage_adapter.clone(),
        );
        return Command {
            protocol,
            ack_build,
            mqtt4_service,
            mqtt5_service,
            metadata_cache,
            response_queue_sx,
            idempotent_manager,
        };
    }

    pub async fn apply(&mut self, connect_id: u64, packet: MQTTPacket) -> Option<MQTTPacket> {
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
                            self.idempotent_manager.clone(),
                        )
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
                                self.idempotent_manager.clone(),
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
                        .publish_ack(pub_ack, pub_ack_properties)
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
                            .un_subscribe(connect_id, unsubscribe, unsubscribe_properties)
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
