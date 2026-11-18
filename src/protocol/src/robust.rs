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

use serde::{Deserialize, Serialize};

use amq_protocol::frame::AMQPFrame;

use crate::{
    kafka::packet::KafkaPacketWrapper,
    mqtt::{
        codec::MqttPacketWrapper,
        common::{MqttPacket, MqttProtocol},
    },
    nats::packet::NatsPacket,
    storage::codec::StorageEnginePacket,
};

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RobustMQProtocol {
    #[default]
    MQTT3,
    MQTT4,
    MQTT5,
    KAFKA,
    AMQP,
    StorageEngine,
    NATS,
}

impl RobustMQProtocol {
    pub fn is_mqtt(&self) -> bool {
        *self == RobustMQProtocol::MQTT3
            || *self == RobustMQProtocol::MQTT4
            || *self == RobustMQProtocol::MQTT5
    }

    pub fn is_mqtt5(&self) -> bool {
        *self == RobustMQProtocol::MQTT5
    }

    pub fn is_kafka(&self) -> bool {
        *self == RobustMQProtocol::KAFKA
    }

    pub fn is_amqp(&self) -> bool {
        *self == RobustMQProtocol::AMQP
    }

    pub fn is_engine(&self) -> bool {
        *self == RobustMQProtocol::StorageEngine
    }

    pub fn is_nats(&self) -> bool {
        *self == RobustMQProtocol::NATS
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            RobustMQProtocol::MQTT3 => 3,
            RobustMQProtocol::MQTT4 => 4,
            RobustMQProtocol::MQTT5 => 5,
            RobustMQProtocol::KAFKA => 0,
            RobustMQProtocol::AMQP => 11,
            RobustMQProtocol::StorageEngine => 10,
            RobustMQProtocol::NATS => 12,
        }
    }

    pub fn to_str(&self) -> String {
        match *self {
            RobustMQProtocol::MQTT3 => "MQTT3".to_string(),
            RobustMQProtocol::MQTT4 => "MQTT4".to_string(),
            RobustMQProtocol::MQTT5 => "MQTT5".to_string(),
            RobustMQProtocol::KAFKA => "KAFKA".to_string(),
            RobustMQProtocol::AMQP => "AMQP".to_string(),
            RobustMQProtocol::StorageEngine => "StorageEngine".to_string(),
            RobustMQProtocol::NATS => "NATS".to_string(),
        }
    }

    pub fn to_mqtt(&self) -> MqttProtocol {
        match *self {
            RobustMQProtocol::MQTT3 => MqttProtocol::Mqtt3,
            RobustMQProtocol::MQTT4 => MqttProtocol::Mqtt4,
            RobustMQProtocol::MQTT5 => MqttProtocol::Mqtt5,
            RobustMQProtocol::KAFKA => MqttProtocol::Mqtt3,
            RobustMQProtocol::AMQP => MqttProtocol::Mqtt3,
            RobustMQProtocol::StorageEngine => MqttProtocol::Mqtt3,
            RobustMQProtocol::NATS => MqttProtocol::Mqtt3,
        }
    }

    pub fn from_u8(protocol: u8) -> RobustMQProtocol {
        match protocol {
            4 => RobustMQProtocol::MQTT4,
            5 => RobustMQProtocol::MQTT5,
            12 => RobustMQProtocol::NATS,
            _ => RobustMQProtocol::MQTT3,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MqttWrapperExtend {
    pub protocol_version: u8,
}

#[derive(Clone, Debug, Default)]
pub struct KafkaWrapperExtend {}

#[derive(Clone, Debug, Default)]
pub struct AmqpWrapperExtend {}

#[derive(Clone, Debug, Default)]
pub struct StorageEngineWrapperExtend {}

#[derive(Clone, Debug, Default)]
pub struct NatsWrapperExtend {}

#[derive(Clone, Debug)]
pub enum RobustMQWrapperExtend {
    MQTT(MqttWrapperExtend),
    KAFKA(KafkaWrapperExtend),
    AMQP(AmqpWrapperExtend),
    StorageEngine(StorageEngineWrapperExtend),
    NATS(NatsWrapperExtend),
}

impl RobustMQWrapperExtend {
    pub fn to_mqtt_protocol(&self) -> u8 {
        match self.clone() {
            RobustMQWrapperExtend::MQTT(extend) => extend.protocol_version,
            RobustMQWrapperExtend::KAFKA(_) => 3,
            RobustMQWrapperExtend::AMQP(_) => 3,
            RobustMQWrapperExtend::StorageEngine(_) => 3,
            RobustMQWrapperExtend::NATS(_) => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RobustMQPacketWrapper {
    pub protocol: RobustMQProtocol,
    pub extend: RobustMQWrapperExtend,
    pub packet: RobustMQPacket,
}

impl RobustMQPacketWrapper {
    pub fn from_mqtt(wrapper: MqttPacketWrapper) -> Self {
        RobustMQPacketWrapper {
            protocol: RobustMQProtocol::from_u8(wrapper.protocol_version),
            extend: RobustMQWrapperExtend::MQTT(MqttWrapperExtend {
                protocol_version: wrapper.protocol_version,
            }),
            packet: RobustMQPacket::MQTT(wrapper.packet),
        }
    }

    pub fn to_mqtt(&self) -> MqttPacketWrapper {
        MqttPacketWrapper {
            protocol_version: self.extend.to_mqtt_protocol(),
            packet: self.packet.get_mqtt_packet().unwrap(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RobustMQPacket {
    MQTT(MqttPacket),
    KAFKA(KafkaPacketWrapper),
    // A logical AMQP event can be more than one wire frame (e.g. Basic.GetOk
    // is followed by its content header and body), so this carries the full
    // sequence to be written back-to-back, in order.
    AMQP(Vec<AMQPFrame>),
    StorageEngine(StorageEnginePacket),
    NATS(NatsPacket),
}

impl RobustMQPacket {
    pub fn get_mqtt_packet(&self) -> Option<MqttPacket> {
        match self.clone() {
            RobustMQPacket::MQTT(pack) => Some(pack),
            _ => None,
        }
    }

    pub fn get_kafka_packet(&self) -> Option<KafkaPacketWrapper> {
        match self.clone() {
            RobustMQPacket::KAFKA(pack) => Some(pack),
            _ => None,
        }
    }

    pub fn get_amqp_packet(&self) -> Option<Vec<AMQPFrame>> {
        match self.clone() {
            RobustMQPacket::AMQP(frames) => Some(frames),
            _ => None,
        }
    }

    pub fn get_nats_packet(&self) -> Option<NatsPacket> {
        match self.clone() {
            RobustMQPacket::NATS(pkt) => Some(pkt),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mqtt::{
        codec::MqttPacketWrapper,
        common::{MqttPacket, PingReq},
    };

    #[test]
    fn protocol_helpers_cover_all_supported_protocols() {
        let protocols = [
            (RobustMQProtocol::MQTT3, 3, "MQTT3"),
            (RobustMQProtocol::MQTT4, 4, "MQTT4"),
            (RobustMQProtocol::MQTT5, 5, "MQTT5"),
            (RobustMQProtocol::KAFKA, 0, "KAFKA"),
            (RobustMQProtocol::AMQP, 11, "AMQP"),
            (RobustMQProtocol::StorageEngine, 10, "StorageEngine"),
            (RobustMQProtocol::NATS, 12, "NATS"),
        ];

        for (protocol, number, name) in protocols {
            assert_eq!(protocol.to_u8(), number);
            assert_eq!(protocol.to_str(), name);
        }

        assert_eq!(RobustMQProtocol::from_u8(4), RobustMQProtocol::MQTT4);
        assert_eq!(RobustMQProtocol::from_u8(5), RobustMQProtocol::MQTT5);
        assert_eq!(RobustMQProtocol::from_u8(12), RobustMQProtocol::NATS);
        assert_eq!(RobustMQProtocol::from_u8(255), RobustMQProtocol::MQTT3);
        assert!(RobustMQProtocol::MQTT5.is_mqtt());
        assert!(RobustMQProtocol::MQTT5.is_mqtt5());
        assert!(RobustMQProtocol::KAFKA.is_kafka());
        assert!(RobustMQProtocol::AMQP.is_amqp());
        assert!(RobustMQProtocol::StorageEngine.is_engine());
        assert!(RobustMQProtocol::NATS.is_nats());
    }

    #[test]
    fn mqtt_packet_wrapper_round_trips_and_non_mqtt_packets_are_filtered() {
        let wrapper = MqttPacketWrapper {
            protocol_version: 5,
            packet: MqttPacket::PingReq(PingReq),
        };
        let robust = RobustMQPacketWrapper::from_mqtt(wrapper.clone());

        assert_eq!(robust.protocol, RobustMQProtocol::MQTT5);
        assert_eq!(robust.to_mqtt().protocol_version, 5);
        assert_eq!(robust.to_mqtt().packet, wrapper.packet);

        let packet = RobustMQPacket::MQTT(wrapper.packet);
        assert!(packet.get_mqtt_packet().is_some());
        assert!(packet.get_kafka_packet().is_none());
        assert!(packet.get_amqp_packet().is_none());
        assert!(packet.get_nats_packet().is_none());

        assert_eq!(
            RobustMQWrapperExtend::NATS(NatsWrapperExtend {}).to_mqtt_protocol(),
            3
        );
    }
}
