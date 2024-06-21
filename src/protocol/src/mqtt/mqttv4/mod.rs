/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::mqtt::common::*;
use std::{str::Utf8Error, slice::Iter, fmt};
use bytes::{Buf, BufMut, Bytes, BytesMut};


pub mod connect;
pub mod connack;
pub mod publish;
pub mod puback;
pub mod pubrec;
pub mod pubrel;
pub mod pubcomp;
pub mod subscribe;
pub mod suback;
pub mod unsubscribe;
pub mod unsuback;
pub mod ping;
pub mod disconnect;
pub mod codec;

#[derive(Debug, Clone)]
pub struct MqttV4;

impl MqttV4 {
    pub fn new() -> Self{
        return Self{};
    }
}

impl Protocol for MqttV4 {
    
     // Reads a stream of bytes and extracts next MQTT packet out of it
     fn read_mut(&mut self, stream: &mut BytesMut, max_size: usize) -> Result<MQTTPacket, Error> {
        let fixed_header = check(stream.iter(), max_size)?;
        // Test with a stream with exactly the size to check border panics
        let packet = stream.split_to(fixed_header.frame_length());
        let packet_type = fixed_header.packet_type()?;

        // if fixed_header.remaining_len == 0 {
        //     // no payload packets
        //     return match packet_type {

        //     }
        // }
        let packet = packet.freeze();
        let packet = match packet_type {
            PacketType::Connect => {
                let (connect, login, lastwill) = connect::read(fixed_header, packet)?;
                MQTTPacket::Connect(connect, None, lastwill, None, login)
            }
            PacketType::ConnAck => MQTTPacket::ConnAck(connack::read(fixed_header, packet)?, None),
            PacketType::Publish => MQTTPacket::Publish(publish::read(fixed_header, packet)?, None),
            PacketType::PubAck => MQTTPacket::PubAck(puback::read(fixed_header, packet)?, None),
            PacketType::PubRec => MQTTPacket::PubRec(pubrec::read(fixed_header, packet)?, None),
            PacketType::PubRel => MQTTPacket::PubRel(pubrel::read(fixed_header, packet)?, None),
            PacketType::PubComp => MQTTPacket::PubComp(pubcomp::read(fixed_header, packet)?, None),
            PacketType::Subscribe => MQTTPacket::Subscribe(subscribe::read(fixed_header, packet)?, None),
            PacketType::SubAck => MQTTPacket::SubAck(suback::read(fixed_header, packet)?, None),
            PacketType::Unsubscribe => MQTTPacket::Unsubscribe(unsubscribe::read(fixed_header, packet)?, None),
            PacketType::UnsubAck => MQTTPacket::UnsubAck(unsuback::read(fixed_header, packet)?, None),
            PacketType::PingReq => MQTTPacket::PingReq(PingReq),
            PacketType::PingResp => MQTTPacket::PingResp(PingResp),
            // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
            // hit when Disconnect packet has properties which are only valid for MQTT V5
            PacketType::Disconnect => return Err(Error::InvalidProtocol),
            _ => unreachable!(),

        };
        Ok(packet)
     }

     fn write(&self, packet: MQTTPacket, buffer: &mut BytesMut) -> Result<usize, Error> {
        let size = match packet {
            MQTTPacket::Connect(connect, None, last_will, None, login) => {
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
        Ok(size)
     }
}

