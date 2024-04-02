use crate::{metadata::cache::MetadataCache, server::MQTTProtocol};
use protocol::mqtt::MQTTPacket;
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
}

impl Command {
    pub fn new(protocol: MQTTProtocol, metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        let ack_build = MQTTAckBuild::new(protocol.clone(), metadata_cache.clone());
        let mqtt4_service = Mqtt4Service::new(metadata_cache.clone(), ack_build.clone());
        let mqtt5_service = Mqtt5Service::new(metadata_cache.clone(), ack_build.clone());
        return Command {
            protocol,
            ack_build,
            mqtt4_service,
            mqtt5_service,
        };
    }

    pub fn apply(&self, packet: MQTTPacket) -> MQTTPacket {
        match packet {
            MQTTPacket::Connect(connect, properties, last_will, last_will_peoperties, login) => {
                if self.protocol == MQTTProtocol::MQTT4 {
                    return self.mqtt4_service.connect(connect, last_will, login);
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self.mqtt5_service.connect(
                        connect,
                        properties,
                        last_will,
                        last_will_peoperties,
                        login,
                    );
                }
            }

            MQTTPacket::Publish(publish, publish_properties) => {
                if self.protocol == MQTTProtocol::MQTT4 {
                    return self.mqtt4_service.publish(publish);
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self.mqtt5_service.publish(publish, publish_properties);
                }
            }

            MQTTPacket::Subscribe(subscribe, subscribe_properties) => {
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
                if self.protocol == MQTTProtocol::MQTT4 {
                    return self.mqtt4_service.ping(ping);
                }

                if self.protocol == MQTTProtocol::MQTT5 {
                    return self.mqtt4_service.ping(ping);
                }
            }

            MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
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
}
