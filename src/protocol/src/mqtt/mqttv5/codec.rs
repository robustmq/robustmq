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
    auth, check, connack, connect, disconnect, ping, puback, pubcomp, publish, pubrec, pubrel,
    suback, subscribe, unsuback, unsubscribe, ConnectReadOutcome, MqttPacket, PacketType,
};

#[derive(Clone, Debug)]
pub struct Mqtt5Codec {}

impl Default for Mqtt5Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl Mqtt5Codec {
    pub fn new() -> Mqtt5Codec {
        Mqtt5Codec {}
    }
}

impl codec::Encoder<MqttPacket> for Mqtt5Codec {
    type Error = super::Error;
    fn encode(&mut self, packet: MqttPacket, buffer: &mut BytesMut) -> Result<(), Self::Error> {
        match packet {
            MqttPacket::Connect(_, connect, properties, last_will, last_will_peoperties, login) => {
                connect::write(
                    &connect,
                    &properties,
                    &last_will,
                    &last_will_peoperties,
                    &login,
                    buffer,
                )?
            }
            MqttPacket::ConnAck(connack, conn_ack_properties) => {
                connack::write(&connack, &conn_ack_properties, buffer)?
            }
            MqttPacket::Publish(publish, publish_properties) => {
                publish::write(&publish, &publish_properties, buffer)?
            }
            MqttPacket::PubAck(puback, pub_ack_properties) => {
                puback::write(&puback, &pub_ack_properties, buffer)?
            }
            MqttPacket::PubRec(pubrec, pub_rec_properties) => {
                pubrec::write(&pubrec, &pub_rec_properties, buffer)?
            }
            MqttPacket::PubRel(pubrel, pub_rel_properties) => {
                pubrel::write(&pubrel, &pub_rel_properties, buffer)?
            }
            MqttPacket::PubComp(pubcomp, pub_comp_properties) => {
                pubcomp::write(&pubcomp, &pub_comp_properties, buffer)?
            }
            MqttPacket::Subscribe(subscribe, subscribe_properties) => {
                subscribe::write(&subscribe, &subscribe_properties, buffer)?
            }
            MqttPacket::SubAck(suback, suback_properties) => {
                suback::write(&suback, &suback_properties, buffer)?
            }
            MqttPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                unsubscribe::write(&unsubscribe, &unsubscribe_properties, buffer)?
            }
            MqttPacket::UnsubAck(unsuback, unsuback_properties) => {
                unsuback::write(&unsuback, &unsuback_properties, buffer)?
            }
            MqttPacket::PingReq(_) => ping::pingreq::write(buffer)?,
            MqttPacket::PingResp(_) => ping::pingresp::write(buffer)?,
            MqttPacket::Disconnect(disconnect, disconnect_properties) => {
                disconnect::write(&disconnect, &disconnect_properties, buffer)?
            }
            MqttPacket::Auth(auth, auth_properties) => {
                auth::write(&auth, &auth_properties, buffer)?
            }
        };
        Ok(())
    }
}

impl codec::Decoder for Mqtt5Codec {
    type Item = MqttPacket;
    type Error = super::Error;
    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let fixed_header = check(stream.iter(), 1000000)?;
        // Test with a stream with exactly the size to check border panics
        let packet = stream.split_to(fixed_header.frame_length());
        let packet_type = fixed_header.packet_type()?;
        let packet = packet.freeze();
        let packet = match packet_type {
            PacketType::Connect => {
                let ConnectReadOutcome {
                    protocol_version,
                    connect,
                    properties,
                    last_will,
                    last_will_properties,
                    login,
                } = connect::read(fixed_header, packet)?;
                MqttPacket::Connect(
                    protocol_version,
                    connect,
                    properties,
                    last_will,
                    last_will_properties,
                    login,
                )
            }
            PacketType::ConnAck => {
                let (conn_ack, conn_ack_properties) = connack::read(fixed_header, packet)?;
                MqttPacket::ConnAck(conn_ack, conn_ack_properties)
            }
            PacketType::Publish => {
                let (publish, publish_properties) = publish::read(fixed_header, packet)?;
                MqttPacket::Publish(publish, publish_properties)
            }
            PacketType::PubAck => {
                let (puback, puback_properties) = puback::read(fixed_header, packet)?;
                MqttPacket::PubAck(puback, puback_properties)
            }
            PacketType::PubRec => {
                let (pubrec, pubrec_properties) = pubrec::read(fixed_header, packet)?;
                MqttPacket::PubRec(pubrec, pubrec_properties)
            }
            PacketType::PubRel => {
                let (pubrel, pubrel_properties) = pubrel::read(fixed_header, packet)?;
                MqttPacket::PubRel(pubrel, pubrel_properties)
            }
            PacketType::PubComp => {
                let (pubcomp, pubcomp_properties) = pubcomp::read(fixed_header, packet)?;
                MqttPacket::PubComp(pubcomp, pubcomp_properties)
            }
            PacketType::Subscribe => {
                let (subscribe, subscribe_properties) = subscribe::read(fixed_header, packet)?;
                MqttPacket::Subscribe(subscribe, subscribe_properties)
            }
            PacketType::SubAck => {
                let (suback, suback_properties) = suback::read(fixed_header, packet)?;
                MqttPacket::SubAck(suback, suback_properties)
            }
            PacketType::Unsubscribe => {
                let (unsubscribe, unsubscribe_properties) =
                    unsubscribe::read(fixed_header, packet)?;
                MqttPacket::Unsubscribe(unsubscribe, unsubscribe_properties)
            }
            PacketType::UnsubAck => {
                let (unsuback, unsuback_properties) = unsuback::read(fixed_header, packet)?;
                MqttPacket::UnsubAck(unsuback, unsuback_properties)
            }
            PacketType::PingReq => MqttPacket::PingReq(super::PingReq),
            PacketType::PingResp => MqttPacket::PingResp(super::PingResp),
            // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
            // hit when Disconnect packet has properties which are only valid for MQTT V5
            PacketType::Disconnect => {
                let (disconnect, disconnect_properties) = disconnect::read(fixed_header, packet)?;
                MqttPacket::Disconnect(disconnect, disconnect_properties)
            }
            PacketType::Auth => {
                let (auth, auth_properties) = auth::read(fixed_header, packet)?;
                MqttPacket::Auth(auth, auth_properties)
            }
        };
        Ok(Some(packet))
    }
}
