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
use common_base::error::mqtt_protocol_error::MQTTProtocolError;
use tokio_util::codec;

use super::common::ConnectReadOutcome;
use crate::mqtt::common::{check, connect_read, MqttPacket, PacketType};

#[derive(Debug, Clone)]
pub struct MqttPacketWrapper {
    pub protocol_version: u8,
    pub packet: MqttPacket,
}

#[derive(Clone, Debug)]
pub struct MqttCodec {
    pub protocol_version: Option<u8>,
}

impl MqttCodec {
    pub fn new(protocol_version: Option<u8>) -> MqttCodec {
        MqttCodec { protocol_version }
    }
}

impl MqttCodec {
    pub fn decode_data(
        &mut self,
        stream: &mut BytesMut,
    ) -> Result<Option<MqttPacket>, MQTTProtocolError> {
        let fixed_header = check(stream.iter(), 1000000)?;
        // Test with a stream with exactly the size to check border panics
        let packet_type = fixed_header.packet_type()?;

        let packet = stream.split_to(fixed_header.frame_length());
        let packet = packet.freeze();

        if packet_type == PacketType::Connect {
            match connect_read(fixed_header, packet.clone()) {
                Ok(ConnectReadOutcome {
                    protocol_version,
                    connect,
                    properties,
                    last_will,
                    last_will_properties,
                    login,
                }) => {
                    self.protocol_version = Some(protocol_version);

                    if protocol_version == 4 || protocol_version == 3 {
                        let packet = MqttPacket::Connect(
                            protocol_version,
                            connect,
                            None,
                            last_will,
                            None,
                            login,
                        );
                        return Ok(Some(packet));
                    }

                    if protocol_version == 5 {
                        let packet = MqttPacket::Connect(
                            protocol_version,
                            connect,
                            properties,
                            last_will,
                            last_will_properties,
                            login,
                        );
                        return Ok(Some(packet));
                    }
                }
                Err(_) => {
                    return Err(MQTTProtocolError::InvalidProtocolName);
                }
            }
        }

        if self.protocol_version.is_none() {
            return Err(MQTTProtocolError::InvalidProtocolName);
        }

        let protocol_version = self.protocol_version.unwrap();

        if protocol_version == 4 || protocol_version == 3 {
            let packet = match packet_type {
                PacketType::ConnAck => MqttPacket::ConnAck(
                    crate::mqtt::mqttv4::connack::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::Publish => MqttPacket::Publish(
                    crate::mqtt::mqttv4::publish::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::PubAck => MqttPacket::PubAck(
                    crate::mqtt::mqttv4::puback::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::PubRec => MqttPacket::PubRec(
                    crate::mqtt::mqttv4::pubrec::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::PubRel => MqttPacket::PubRel(
                    crate::mqtt::mqttv4::pubrel::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::PubComp => MqttPacket::PubComp(
                    crate::mqtt::mqttv4::pubcomp::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::Subscribe => MqttPacket::Subscribe(
                    crate::mqtt::mqttv4::subscribe::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::SubAck => MqttPacket::SubAck(
                    crate::mqtt::mqttv4::suback::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::Unsubscribe => MqttPacket::Unsubscribe(
                    crate::mqtt::mqttv4::unsubscribe::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::UnsubAck => MqttPacket::UnsubAck(
                    crate::mqtt::mqttv4::unsuback::read(fixed_header, packet)?,
                    None,
                ),
                PacketType::PingReq => MqttPacket::PingReq(crate::mqtt::common::PingReq),
                PacketType::PingResp => MqttPacket::PingResp(crate::mqtt::common::PingResp),
                // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
                // hit when Disconnect packet has properties which are only valid for MQTT V5
                // PacketType::Disconnect => return Err(Error::InvalidProtocol),
                PacketType::Disconnect => {
                    let (disconnect, _) =
                        crate::mqtt::mqttv5::disconnect::read(fixed_header, packet)?;
                    MqttPacket::Disconnect(disconnect, None)
                }
                _ => unreachable!(),
            };
            return Ok(Some(packet));
        } else if protocol_version == 5 {
            let packet = match packet_type {
                PacketType::ConnAck => {
                    let (conn_ack, conn_ack_properties) =
                        crate::mqtt::mqttv5::connack::read(fixed_header, packet)?;
                    MqttPacket::ConnAck(conn_ack, conn_ack_properties)
                }
                PacketType::Publish => {
                    let (publish, publish_properties) =
                        crate::mqtt::mqttv5::publish::read(fixed_header, packet)?;
                    MqttPacket::Publish(publish, publish_properties)
                }
                PacketType::PubAck => {
                    let (puback, puback_properties) =
                        crate::mqtt::mqttv5::puback::read(fixed_header, packet)?;
                    MqttPacket::PubAck(puback, puback_properties)
                }
                PacketType::PubRec => {
                    let (pubrec, pubrec_properties) =
                        crate::mqtt::mqttv5::pubrec::read(fixed_header, packet)?;
                    MqttPacket::PubRec(pubrec, pubrec_properties)
                }
                PacketType::PubRel => {
                    let (pubrel, pubrel_properties) =
                        crate::mqtt::mqttv5::pubrel::read(fixed_header, packet)?;
                    MqttPacket::PubRel(pubrel, pubrel_properties)
                }
                PacketType::PubComp => {
                    let (pubcomp, pubcomp_properties) =
                        crate::mqtt::mqttv5::pubcomp::read(fixed_header, packet)?;
                    MqttPacket::PubComp(pubcomp, pubcomp_properties)
                }
                PacketType::Subscribe => {
                    let (subscribe, subscribe_properties) =
                        crate::mqtt::mqttv5::subscribe::read(fixed_header, packet)?;
                    MqttPacket::Subscribe(subscribe, subscribe_properties)
                }
                PacketType::SubAck => {
                    let (suback, suback_properties) =
                        crate::mqtt::mqttv5::suback::read(fixed_header, packet)?;
                    MqttPacket::SubAck(suback, suback_properties)
                }
                PacketType::Unsubscribe => {
                    let (unsubscribe, unsubscribe_properties) =
                        crate::mqtt::mqttv5::unsubscribe::read(fixed_header, packet)?;
                    MqttPacket::Unsubscribe(unsubscribe, unsubscribe_properties)
                }
                PacketType::UnsubAck => {
                    let (unsuback, unsuback_properties) =
                        crate::mqtt::mqttv5::unsuback::read(fixed_header, packet)?;
                    MqttPacket::UnsubAck(unsuback, unsuback_properties)
                }
                PacketType::PingReq => MqttPacket::PingReq(crate::mqtt::common::PingReq),
                PacketType::PingResp => MqttPacket::PingResp(crate::mqtt::common::PingResp),
                // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
                // hit when Disconnect packet has properties which are only valid for MQTT V5
                PacketType::Disconnect => {
                    let (disconnect, disconnect_properties) =
                        crate::mqtt::mqttv5::disconnect::read(fixed_header, packet)?;
                    MqttPacket::Disconnect(disconnect, disconnect_properties)
                }
                _ => unreachable!(),
            };
            return Ok(Some(packet));
        }

        Err(MQTTProtocolError::InvalidProtocolName)
    }

    pub fn encode_data(
        &mut self,
        packet_wrapper: MqttPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), MQTTProtocolError> {
        let packet = packet_wrapper.packet;
        let protocol_version = packet_wrapper.protocol_version;

        if protocol_version == 4 || protocol_version == 3 {
            match packet {
                MqttPacket::Connect(_,connect, None, last_will, None, login) => {
                    crate::mqtt::mqttv4::connect::write(&connect, &login, &last_will, buffer)?
                }
                MqttPacket::ConnAck(connack, _) => crate::mqtt::mqttv4::connack::write(&connack, buffer)?,
                MqttPacket::Publish(publish, None) => crate::mqtt::mqttv4::publish::write(&publish, buffer)?,
                MqttPacket::PubAck(puback, None) => crate::mqtt::mqttv4::puback::write(&puback, buffer)?,
                MqttPacket::PubRec(pubrec, None) => crate::mqtt::mqttv4::pubrec::write(&pubrec, buffer)?,
                MqttPacket::PubRel(pubrel, None) => crate::mqtt::mqttv4::pubrel::write(&pubrel, buffer)?,
                MqttPacket::PubComp(pubcomp, None) => crate::mqtt::mqttv4::pubcomp::write(&pubcomp, buffer)?,
                MqttPacket::Subscribe(subscribe, None) => crate::mqtt::mqttv4::subscribe::write(&subscribe, buffer)?,
                MqttPacket::SubAck(suback, None) => crate::mqtt::mqttv4::suback::write(&suback, buffer)?,
                MqttPacket::Unsubscribe(unsubscribe, None) => crate::mqtt::mqttv4::unsubscribe::write(&unsubscribe, buffer)?,
                MqttPacket::UnsubAck(unsuback, None) => crate::mqtt::mqttv4::unsuback::write(&unsuback, buffer)?,
                MqttPacket::PingReq(_) => crate::mqtt::mqttv4::ping::pingreq::write(buffer)?,
                MqttPacket::PingResp(_) => crate::mqtt::mqttv4::ping::pingresp::write(buffer)?,
                MqttPacket::Disconnect(disconnect, None) => crate::mqtt::mqttv4::disconnect::write(&disconnect, buffer)?,

                //Packet::
                _=> unreachable!(
                    "This branch only matches for packets with Properties, which is not possible in MQTT V4",
                ),
            };
        } else if protocol_version == 5 {
            match packet {
                MqttPacket::Connect(
                    _,
                    connect,
                    properties,
                    last_will,
                    last_will_peoperties,
                    login,
                ) => crate::mqtt::mqttv5::connect::write(
                    &connect,
                    &properties,
                    &last_will,
                    &last_will_peoperties,
                    &login,
                    buffer,
                )?,
                MqttPacket::ConnAck(connack, conn_ack_properties) => {
                    crate::mqtt::mqttv5::connack::write(&connack, &conn_ack_properties, buffer)?
                }
                MqttPacket::Publish(publish, publish_properties) => {
                    crate::mqtt::mqttv5::publish::write(&publish, &publish_properties, buffer)?
                }
                MqttPacket::PubAck(puback, pub_ack_properties) => {
                    crate::mqtt::mqttv5::puback::write(&puback, &pub_ack_properties, buffer)?
                }
                MqttPacket::PubRec(pubrec, pub_rec_properties) => {
                    crate::mqtt::mqttv5::pubrec::write(&pubrec, &pub_rec_properties, buffer)?
                }
                MqttPacket::PubRel(pubrel, pub_rel_properties) => {
                    crate::mqtt::mqttv5::pubrel::write(&pubrel, &pub_rel_properties, buffer)?
                }
                MqttPacket::PubComp(pubcomp, pub_comp_properties) => {
                    crate::mqtt::mqttv5::pubcomp::write(&pubcomp, &pub_comp_properties, buffer)?
                }
                MqttPacket::Subscribe(subscribe, subscribe_properties) => {
                    crate::mqtt::mqttv5::subscribe::write(
                        &subscribe,
                        &subscribe_properties,
                        buffer,
                    )?
                }
                MqttPacket::SubAck(suback, suback_properties) => {
                    crate::mqtt::mqttv5::suback::write(&suback, &suback_properties, buffer)?
                }
                MqttPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                    crate::mqtt::mqttv5::unsubscribe::write(
                        &unsubscribe,
                        &unsubscribe_properties,
                        buffer,
                    )?
                }
                MqttPacket::UnsubAck(unsuback, unsuback_properties) => {
                    crate::mqtt::mqttv5::unsuback::write(&unsuback, &unsuback_properties, buffer)?
                }
                MqttPacket::PingReq(_) => crate::mqtt::mqttv5::ping::pingreq::write(buffer)?,
                MqttPacket::PingResp(_) => crate::mqtt::mqttv5::ping::pingresp::write(buffer)?,
                MqttPacket::Disconnect(disconnect, disconnect_properties) => {
                    crate::mqtt::mqttv5::disconnect::write(
                        &disconnect,
                        &disconnect_properties,
                        buffer,
                    )?
                }
                MqttPacket::Auth(auth, auth_properties) => {
                    crate::mqtt::mqttv5::auth::write(&auth, &auth_properties, buffer)?
                }
            };
        }
        Ok(())
    }
}

impl codec::Encoder<MqttPacketWrapper> for MqttCodec {
    type Error = MQTTProtocolError;
    fn encode(
        &mut self,
        packet_wrapper: MqttPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        self.encode_data(packet_wrapper, buffer)
    }
}

impl codec::Decoder for MqttCodec {
    type Item = MqttPacket;
    type Error = MQTTProtocolError;
    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_data(stream)
    }
}

pub fn calc_mqtt_packet_size(packet_wrapper: MqttPacketWrapper) -> usize {
    calc_mqtt_packet_len(packet_wrapper).unwrap_or_default()
}

pub fn calc_mqtt_packet_len(packet_wrapper: MqttPacketWrapper) -> Result<usize, MQTTProtocolError> {
    let packet = packet_wrapper.packet;
    let protocol_version = packet_wrapper.protocol_version;
    let mut buffer = BytesMut::new();
    let mut size = 0;
    if protocol_version == 4 || protocol_version == 3 {
        size = match packet {
            MqttPacket::Connect(_,connect, None, last_will, None, login) => {
                crate::mqtt::mqttv4::connect::write(&connect, &login, &last_will, &mut buffer)?
            }
            MqttPacket::ConnAck(connack, _) => crate::mqtt::mqttv4::connack::write(&connack, &mut buffer)?,
            MqttPacket::Publish(publish, None) => crate::mqtt::mqttv4::publish::write(&publish, &mut buffer)?,
            MqttPacket::PubAck(puback, None) => crate::mqtt::mqttv4::puback::write(&puback, &mut buffer)?,
            MqttPacket::PubRec(pubrec, None) => crate::mqtt::mqttv4::pubrec::write(&pubrec, &mut buffer)?,
            MqttPacket::PubRel(pubrel, None) => crate::mqtt::mqttv4::pubrel::write(&pubrel, &mut buffer)?,
            MqttPacket::PubComp(pubcomp, None) => crate::mqtt::mqttv4::pubcomp::write(&pubcomp, &mut buffer)?,
            MqttPacket::Subscribe(subscribe, None) => crate::mqtt::mqttv4::subscribe::write(&subscribe, &mut buffer)?,
            MqttPacket::SubAck(suback, None) => crate::mqtt::mqttv4::suback::write(&suback, &mut buffer)?,
            MqttPacket::Unsubscribe(unsubscribe, None) => crate::mqtt::mqttv4::unsubscribe::write(&unsubscribe, &mut buffer)?,
            MqttPacket::UnsubAck(unsuback, None) => crate::mqtt::mqttv4::unsuback::write(&unsuback, &mut buffer)?,
            MqttPacket::PingReq(_) => crate::mqtt::mqttv4::ping::pingreq::write(&mut buffer)?,
            MqttPacket::PingResp(_) => crate::mqtt::mqttv4::ping::pingresp::write(&mut buffer)?,
            MqttPacket::Disconnect(disconnect, None) => crate::mqtt::mqttv4::disconnect::write(&disconnect, &mut buffer)?,

            //Packet::
            _=> unreachable!(
                "This branch only matches for packets with Properties, which is not possible in MQTT V4,{:?}",packet
            ),
        };
    } else if protocol_version == 5 {
        size = match packet {
            MqttPacket::Connect(_, connect, properties, last_will, last_will_peoperties, login) => {
                crate::mqtt::mqttv5::connect::write(
                    &connect,
                    &properties,
                    &last_will,
                    &last_will_peoperties,
                    &login,
                    &mut buffer,
                )?
            }
            MqttPacket::ConnAck(connack, conn_ack_properties) => {
                crate::mqtt::mqttv5::connack::write(&connack, &conn_ack_properties, &mut buffer)?
            }
            MqttPacket::Publish(publish, publish_properties) => {
                crate::mqtt::mqttv5::publish::write(&publish, &publish_properties, &mut buffer)?
            }
            MqttPacket::PubAck(puback, pub_ack_properties) => {
                crate::mqtt::mqttv5::puback::write(&puback, &pub_ack_properties, &mut buffer)?
            }
            MqttPacket::PubRec(pubrec, pub_rec_properties) => {
                crate::mqtt::mqttv5::pubrec::write(&pubrec, &pub_rec_properties, &mut buffer)?
            }
            MqttPacket::PubRel(pubrel, pub_rel_properties) => {
                crate::mqtt::mqttv5::pubrel::write(&pubrel, &pub_rel_properties, &mut buffer)?
            }
            MqttPacket::PubComp(pubcomp, pub_comp_properties) => {
                crate::mqtt::mqttv5::pubcomp::write(&pubcomp, &pub_comp_properties, &mut buffer)?
            }
            MqttPacket::Subscribe(subscribe, subscribe_properties) => {
                crate::mqtt::mqttv5::subscribe::write(
                    &subscribe,
                    &subscribe_properties,
                    &mut buffer,
                )?
            }
            MqttPacket::SubAck(suback, suback_properties) => {
                crate::mqtt::mqttv5::suback::write(&suback, &suback_properties, &mut buffer)?
            }
            MqttPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                crate::mqtt::mqttv5::unsubscribe::write(
                    &unsubscribe,
                    &unsubscribe_properties,
                    &mut buffer,
                )?
            }
            MqttPacket::UnsubAck(unsuback, unsuback_properties) => {
                crate::mqtt::mqttv5::unsuback::write(&unsuback, &unsuback_properties, &mut buffer)?
            }
            MqttPacket::PingReq(_) => crate::mqtt::mqttv5::ping::pingreq::write(&mut buffer)?,
            MqttPacket::PingResp(_) => crate::mqtt::mqttv5::ping::pingresp::write(&mut buffer)?,
            MqttPacket::Disconnect(disconnect, disconnect_properties) => {
                crate::mqtt::mqttv5::disconnect::write(
                    &disconnect,
                    &disconnect_properties,
                    &mut buffer,
                )?
            }
            MqttPacket::Auth(auth, auth_properties) => {
                crate::mqtt::mqttv5::auth::write(&auth, &auth_properties, &mut buffer)?
            }
        };
    }
    Ok(size)
}

pub fn parse_mqtt_packet_to_name(packet: MqttPacket) -> String {
    let name = match packet {
        MqttPacket::Connect(_, _, _, _, _, _) => "connect",
        MqttPacket::ConnAck(_, _) => "conn_ack",
        MqttPacket::Publish(_, _) => "publish",
        MqttPacket::PubAck(_, _) => "pub_ack",
        MqttPacket::PubRec(_, _) => "pub_rec",
        MqttPacket::PubRel(_, _) => "pub_rel",
        MqttPacket::PubComp(_, _) => "pub_comp",
        MqttPacket::Subscribe(_, _) => "subscribe",
        MqttPacket::SubAck(_, _) => "sub_ack",
        MqttPacket::Unsubscribe(_, _) => "sub_ack",
        MqttPacket::UnsubAck(_, _) => "unsub_ack",
        MqttPacket::PingReq(_) => "ping",
        MqttPacket::PingResp(_) => "pong",
        MqttPacket::Disconnect(_, _) => "disconnect",
        MqttPacket::Auth(_, _) => "auth",
    };
    name.to_string()
}
