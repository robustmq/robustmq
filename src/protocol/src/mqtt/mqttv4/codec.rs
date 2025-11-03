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

use super::{
    check, connack, connect, disconnect, ping, puback, pubcomp, publish, pubrec, pubrel, suback,
    subscribe, unsuback, unsubscribe, MqttPacket, PacketType,
};
use bytes::BytesMut;
use common_base::error::mqtt_protocol_error::MQTTProtocolError;
use tokio_util::codec;

#[derive(Clone, Debug)]
pub struct Mqtt4Codec {}

impl Default for Mqtt4Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl Mqtt4Codec {
    pub fn new() -> Mqtt4Codec {
        Mqtt4Codec {}
    }
}

impl codec::Encoder<MqttPacket> for Mqtt4Codec {
    type Error = MQTTProtocolError;
    fn encode(&mut self, packet: MqttPacket, buffer: &mut BytesMut) -> Result<(), Self::Error> {
        match packet {
            MqttPacket::Connect(_,connect, None, last_will, None, login) => {
                connect::write(&connect, &login, &last_will, buffer)?
            }
            MqttPacket::ConnAck(connack, _) => connack::write(&connack, buffer)?,
            MqttPacket::Publish(publish, None) => publish::write(&publish, buffer)?,
            MqttPacket::PubAck(puback, None) => puback::write(&puback, buffer)?,
            MqttPacket::PubRec(pubrec, None) => pubrec::write(&pubrec, buffer)?,
            MqttPacket::PubRel(pubrel, None) => pubrel::write(&pubrel, buffer)?,
            MqttPacket::PubComp(pubcomp, None) => pubcomp::write(&pubcomp, buffer)?,
            MqttPacket::Subscribe(subscribe, None) => subscribe::write(&subscribe, buffer)?,
            MqttPacket::SubAck(suback, None) => suback::write(&suback, buffer)?,
            MqttPacket::Unsubscribe(unsubscribe, None) => unsubscribe::write(&unsubscribe, buffer)?,
            MqttPacket::UnsubAck(unsuback, None) => unsuback::write(&unsuback, buffer)?,
            MqttPacket::PingReq(_) => ping::pingreq::write(buffer)?,
            MqttPacket::PingResp(_) => ping::pingresp::write(buffer)?,
            MqttPacket::Disconnect(disconnect, None) => disconnect::write(&disconnect, buffer)?,

            //Packet::
            _=> unreachable!(
                "This branch only matches for packets with Properties, which is not possible in MQTT V4",
            ),
        };
        Ok(())
    }
}

impl codec::Decoder for Mqtt4Codec {
    type Item = MqttPacket;
    type Error = MQTTProtocolError;
    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let fixed_header = check(stream.iter(), 1000000)?;
        // Test with a stream with exactly the size to check border panics
        let packet = stream.split_to(fixed_header.frame_length());
        let packet_type = fixed_header.packet_type()?;
        let packet = packet.freeze();
        let packet = match packet_type {
            PacketType::Connect => {
                let (protocol_level, connect, login, lastwill) =
                    connect::read(fixed_header, packet)?;
                MqttPacket::Connect(protocol_level, connect, None, lastwill, None, login)
            }
            PacketType::ConnAck => MqttPacket::ConnAck(connack::read(fixed_header, packet)?, None),
            PacketType::Publish => MqttPacket::Publish(publish::read(fixed_header, packet)?, None),
            PacketType::PubAck => MqttPacket::PubAck(puback::read(fixed_header, packet)?, None),
            PacketType::PubRec => MqttPacket::PubRec(pubrec::read(fixed_header, packet)?, None),
            PacketType::PubRel => MqttPacket::PubRel(pubrel::read(fixed_header, packet)?, None),
            PacketType::PubComp => MqttPacket::PubComp(pubcomp::read(fixed_header, packet)?, None),
            PacketType::Subscribe => {
                MqttPacket::Subscribe(subscribe::read(fixed_header, packet)?, None)
            }
            PacketType::SubAck => MqttPacket::SubAck(suback::read(fixed_header, packet)?, None),
            PacketType::Unsubscribe => {
                MqttPacket::Unsubscribe(unsubscribe::read(fixed_header, packet)?, None)
            }
            PacketType::UnsubAck => {
                MqttPacket::UnsubAck(unsuback::read(fixed_header, packet)?, None)
            }
            PacketType::PingReq => MqttPacket::PingReq(super::PingReq),
            PacketType::PingResp => MqttPacket::PingResp(super::PingResp),
            // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
            // hit when Disconnect packet has properties which are only valid for MQTT V5
            PacketType::Disconnect => return Err(MQTTProtocolError::InvalidProtocolName),
            PacketType::Auth => return Err(MQTTProtocolError::InvalidProtocolName),
        };
        Ok(Some(packet))
    }
}
