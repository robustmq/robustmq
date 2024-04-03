use crate::storage::storage::StorageLayer;
use crate::{metadata::cache::MetadataCache, server::MQTTProtocol};
use common_base::log::info;
use protocol::mqtt::{ConnectReturnCode, MQTTPacket};
use std::sync::{Arc, RwLock};

use super::mqtt4::Mqtt4Service;
use super::mqtt5::Mqtt5Service;
use super::packet::MQTTAckBuild;

#[derive(Clone)]
pub struct Command {
    protocol: MQTTProtocol,
    ack_build: MQTTAckBuild,
    mqtt4_service: Mqtt4Service,
    mqtt5_service: Mqtt5Service,
    un_login: bool,
    storage_layer: StorageLayer,
}

impl Command {
    pub fn new(protocol: MQTTProtocol, metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        let ack_build = MQTTAckBuild::new(protocol.clone(), metadata_cache.clone());
        let mqtt4_service = Mqtt4Service::new(metadata_cache.clone(), ack_build.clone());
        let mqtt5_service = Mqtt5Service::new(metadata_cache.clone(), ack_build.clone());
        let storage_layer = StorageLayer::new();
        return Command {
            protocol,
            ack_build,
            mqtt4_service,
            mqtt5_service,
            un_login: true,
            storage_layer,
        };
    }

    pub fn apply(&mut self, packet: MQTTPacket) -> MQTTPacket {
        info(format!("revc packet:{:?}", packet));
        match packet {
            MQTTPacket::Connect(connect, properties, last_will, last_will_peoperties, login) => {
                let mut ack_pkg = self
                    .ack_build
                    .distinct(protocol::mqtt::DisconnectReasonCode::NotAuthorized);

                if self.protocol == MQTTProtocol::MQTT4 {
                    ack_pkg = self.mqtt4_service.connect(
                        connect.clone(),
                        last_will.clone(),
                        login.clone(),
                    );
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    ack_pkg = self.mqtt5_service.connect(
                        connect,
                        properties,
                        last_will,
                        last_will_peoperties,
                        login,
                    );
                }

                if let MQTTPacket::ConnAck(conn_ack, _) = ack_pkg.clone() {
                    if conn_ack.code == ConnectReturnCode::Success {
                        self.un_login = false;
                    }
                }

                return ack_pkg;
            }

            MQTTPacket::Publish(publish, publish_properties) => {
                if self.un_login {
                    return self.un_login_err();
                }

                if self.protocol == MQTTProtocol::MQTT4 {
                    return self.mqtt4_service.publish(publish);
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self.mqtt5_service.publish(publish, publish_properties);
                }
            }

            MQTTPacket::Subscribe(subscribe, subscribe_properties) => {
                if self.un_login {
                    return self.un_login_err();
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return self.mqtt4_service.subscribe(subscribe);
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self
                        .mqtt5_service
                        .subscribe(subscribe, subscribe_properties);
                }
            }

            MQTTPacket::PingReq(ping) => {
                if self.un_login {
                    return self.un_login_err();
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return self.mqtt4_service.ping(ping);
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self.mqtt4_service.ping(ping);
                }
            }

            MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                if self.un_login {
                    return self.un_login_err();
                }
                if self.protocol == MQTTProtocol::MQTT4 {
                    return self.mqtt4_service.un_subscribe(unsubscribe);
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self
                        .mqtt5_service
                        .un_subscribe(unsubscribe, unsubscribe_properties);
                }
            }

            _ => {
                return self
                    .ack_build
                    .distinct(protocol::mqtt::DisconnectReasonCode::ImplementationSpecificError);
            }
        }
        return self
            .ack_build
            .distinct(protocol::mqtt::DisconnectReasonCode::ImplementationSpecificError);
    }

    fn un_login_err(&self) -> MQTTPacket {
        return self
            .ack_build
            .distinct(protocol::mqtt::DisconnectReasonCode::NotAuthorized);
    }
}
