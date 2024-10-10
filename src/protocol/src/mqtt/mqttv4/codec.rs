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

use bytes::BytesMut;
use tokio_util::codec;

use super::{
    check, connack, connect, disconnect, ping, puback, pubcomp, publish, pubrec, pubrel, suback,
    subscribe, unsuback, unsubscribe, Error, MQTTPacket, PacketType,
};

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

impl codec::Encoder<MQTTPacket> for Mqtt4Codec {
    type Error = super::Error;
    fn encode(&mut self, packet: MQTTPacket, buffer: &mut BytesMut) -> Result<(), Self::Error> {
        match packet {
            MQTTPacket::Connect(protocol_version,connect, None, last_will, None, login) => {
                connect::write(&connect, &login, &last_will, buffer)?
            }
            MQTTPacket::ConnAck(connack, _) => connack::write(&connack, buffer)?,
            MQTTPacket::Publish(publish, None) => publish::write(&publish, buffer)?,
            MQTTPacket::PubAck(puback, None) => puback::write(&puback, buffer)?,
            MQTTPacket::PubRec(pubrec, None) => pubrec::write(&pubrec, buffer)?,
            MQTTPacket::PubRel(pubrel, None) => pubrel::write(&pubrel, buffer)?,
            MQTTPacket::PubComp(pubcomp, None) => pubcomp::write(&pubcomp, buffer)?,
            MQTTPacket::Subscribe(subscribe, None) => subscribe::write(&subscribe, buffer)?,
            MQTTPacket::SubAck(suback, None) => suback::write(&suback, buffer)?,
            MQTTPacket::Unsubscribe(unsubscribe, None) => unsubscribe::write(&unsubscribe, buffer)?,
            MQTTPacket::UnsubAck(unsuback, None) => unsuback::write(&unsuback, buffer)?,
            MQTTPacket::PingReq(pingreq) => ping::pingreq::write(buffer)?,
            MQTTPacket::PingResp(pingresp) => ping::pingresp::write(buffer)?,
            MQTTPacket::Disconnect(disconnect, None) => disconnect::write(&disconnect, buffer)?,

            //Packet::
            _=> unreachable!(
                "This branch only matches for packets with Properties, which is not possible in MQTT V4",
            ),
        };
        Ok(())
    }
}

impl codec::Decoder for Mqtt4Codec {
    type Item = MQTTPacket;
    type Error = super::Error;
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
                MQTTPacket::Connect(protocol_level, connect, None, lastwill, None, login)
            }
            PacketType::ConnAck => MQTTPacket::ConnAck(connack::read(fixed_header, packet)?, None),
            PacketType::Publish => MQTTPacket::Publish(publish::read(fixed_header, packet)?, None),
            PacketType::PubAck => MQTTPacket::PubAck(puback::read(fixed_header, packet)?, None),
            PacketType::PubRec => MQTTPacket::PubRec(pubrec::read(fixed_header, packet)?, None),
            PacketType::PubRel => MQTTPacket::PubRel(pubrel::read(fixed_header, packet)?, None),
            PacketType::PubComp => MQTTPacket::PubComp(pubcomp::read(fixed_header, packet)?, None),
            PacketType::Subscribe => {
                MQTTPacket::Subscribe(subscribe::read(fixed_header, packet)?, None)
            }
            PacketType::SubAck => MQTTPacket::SubAck(suback::read(fixed_header, packet)?, None),
            PacketType::Unsubscribe => {
                MQTTPacket::Unsubscribe(unsubscribe::read(fixed_header, packet)?, None)
            }
            PacketType::UnsubAck => {
                MQTTPacket::UnsubAck(unsuback::read(fixed_header, packet)?, None)
            }
            PacketType::PingReq => MQTTPacket::PingReq(super::PingReq),
            PacketType::PingResp => MQTTPacket::PingResp(super::PingResp),
            // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
            // hit when Disconnect packet has properties which are only valid for MQTT V5
            PacketType::Disconnect => return Err(Error::InvalidProtocol),
            _ => unreachable!(),
        };
        Ok(Some(packet))
    }
}
