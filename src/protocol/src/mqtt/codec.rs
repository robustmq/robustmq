use crate::mqtt::common::{check, connect_read, Error, LastWillProperties, MQTTPacket, PacketType};
use bytes::BytesMut;
use tokio_util::codec;

#[derive(Debug,Clone)]
pub struct MQTTPacketWrapper{
    pub protocol_version: u8,
    pub packet:MQTTPacket
}

#[derive(Clone, Debug)]
pub struct MqttCodec {
    pub protocol_version: Option<u8>,
}

impl MqttCodec {
    pub fn new(protocol_version:Option<u8>) -> MqttCodec {
        return MqttCodec {
            protocol_version: None,
        };
    }
}

impl codec::Encoder<MQTTPacketWrapper> for MqttCodec {
    type Error = crate::mqtt::common::Error;
    fn encode(
        &mut self,
        packet_wrapper: MQTTPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let packet = packet_wrapper.packet;
        let protocol_version = packet_wrapper.protocol_version;

        if protocol_version == 4 || protocol_version == 3{
            let size = match packet {
                MQTTPacket::Connect(protocol_version,connect, None, last_will, None, login) => {
                    crate::mqtt::mqttv4::connect::write(&connect, &login, &last_will, buffer)?
                }
                MQTTPacket::ConnAck(connack, _) => crate::mqtt::mqttv4::connack::write(&connack, buffer)?,
                MQTTPacket::Publish(publish, None) => crate::mqtt::mqttv4::publish::write(&publish, buffer)?,
                MQTTPacket::PubAck(puback, None) => crate::mqtt::mqttv4::puback::write(&puback, buffer)?,
                MQTTPacket::PubRec(pubrec, None) => crate::mqtt::mqttv4::pubrec::write(&pubrec, buffer)?,
                MQTTPacket::PubRel(pubrel, None) => crate::mqtt::mqttv4::pubrel::write(&pubrel, buffer)?,
                MQTTPacket::PubComp(pubcomp, None) => crate::mqtt::mqttv4::pubcomp::write(&pubcomp, buffer)?,
                MQTTPacket::Subscribe(subscribe, None) => crate::mqtt::mqttv4::subscribe::write(&subscribe, buffer)?,
                MQTTPacket::SubAck(suback, None) => crate::mqtt::mqttv4::suback::write(&suback, buffer)?,
                MQTTPacket::Unsubscribe(unsubscribe, None) => crate::mqtt::mqttv4::unsubscribe::write(&unsubscribe, buffer)?,
                MQTTPacket::UnsubAck(unsuback, None) => crate::mqtt::mqttv4::unsuback::write(&unsuback, buffer)?,
                MQTTPacket::PingReq(pingreq) => crate::mqtt::mqttv4::ping::pingreq::write(buffer)?,
                MQTTPacket::PingResp(pingresp) => crate::mqtt::mqttv4::ping::pingresp::write(buffer)?,
                MQTTPacket::Disconnect(disconnect, None) => crate::mqtt::mqttv4::disconnect::write(&disconnect, buffer)?,
    
                //Packet::
                _=> unreachable!(
                    "This branch only matches for packets with Properties, which is not possible in MQTT V4",
                ),
            };
        }else if protocol_version == 5 {
            let size = match packet {
                MQTTPacket::Connect(protocol_version,connect, properties, last_will, last_will_peoperties, login) => {
                    crate::mqtt::mqttv5::connect::write(&connect, &properties, &last_will,  &last_will_peoperties, &login, buffer)?
                }
                MQTTPacket::ConnAck(connack, conn_ack_properties) => crate::mqtt::mqttv5::connack::write(&connack,&conn_ack_properties, buffer)?,
                MQTTPacket::Publish(publish, publish_properties ) => crate::mqtt::mqttv5::publish::write(&publish, &publish_properties,buffer)?,
                MQTTPacket::PubAck(puback, pub_ack_properties) => crate::mqtt::mqttv5::puback::write(&puback, &pub_ack_properties,buffer)?,
                MQTTPacket::PubRec(pubrec, pub_rec_properties) => crate::mqtt::mqttv5::pubrec::write(&pubrec, &pub_rec_properties,buffer)?,
                MQTTPacket::PubRel(pubrel, pub_rel_properties) => crate::mqtt::mqttv5::pubrel::write(&pubrel, &pub_rel_properties,buffer)?,
                MQTTPacket::PubComp(pubcomp, pub_comp_properties) => crate::mqtt::mqttv5::pubcomp::write(&pubcomp, &pub_comp_properties,buffer)?,
                MQTTPacket::Subscribe(subscribe, subscribe_properties) => crate::mqtt::mqttv5::subscribe::write(&subscribe,&subscribe_properties, buffer)?,
                MQTTPacket::SubAck(suback, suback_properties) => crate::mqtt::mqttv5::suback::write(&suback, &suback_properties,buffer)?,
                MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => crate::mqtt::mqttv5::unsubscribe::write(&unsubscribe, &unsubscribe_properties,buffer)?,
                MQTTPacket::UnsubAck(unsuback, unsuback_properties) => crate::mqtt::mqttv5::unsuback::write(&unsuback, &unsuback_properties,buffer)?,
                MQTTPacket::PingReq(pingreq) => crate::mqtt::mqttv5::ping::pingreq::write(buffer)?,
                MQTTPacket::PingResp(pingresp) => crate::mqtt::mqttv5::ping::pingresp::write(buffer)?,
                MQTTPacket::Disconnect(disconnect, disconnect_properties) => crate::mqtt::mqttv5::disconnect::write(&disconnect, &disconnect_properties,buffer)?,
    
                //Packet::
                _=> unreachable!(
                    "This branch only matches for packets with Properties, which is not possible in MQTT V4",
                ),
            };
        }
        Ok(())
    }
}

impl codec::Decoder for MqttCodec {
    type Item = MQTTPacket;
    type Error = crate::mqtt::common::Error;
    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let fixed_header = check(stream.iter(), 1000000)?;
        // Test with a stream with exactly the size to check border panics
        let packet = stream.split_to(fixed_header.frame_length());
        let packet_type = fixed_header.packet_type()?;
        let packet = packet.freeze();
        
        if packet_type == PacketType::Connect{
            match  connect_read(fixed_header, packet.clone()){
                Ok((protocol_version,connect, properties, last_will, last_will_properties, login)) => {
                    self.protocol_version = Some(protocol_version);

                    println!("xxx decode {:?}",self.protocol_version);
                    if protocol_version == 4 || protocol_version == 3{
                        let packet = MQTTPacket::Connect(protocol_version,connect, None, last_will, None, login);
                        return Ok(Some(packet));
                    }
        
                    if protocol_version == 5{
                        let packet = MQTTPacket::Connect(protocol_version, connect, properties, last_will, last_will_properties, login);
                        return Ok(Some(packet));
                    }
                }
                Err(e) => {
                    println!("{}",e.to_string());
                    return Err(Error::InvalidProtocol);
                }
            }
            
        }

        if self.protocol_version.is_none(){
            return Err(Error::InvalidProtocol);
        }

        let protocol_version = self.protocol_version.unwrap();

        if protocol_version == 4 || protocol_version == 3{
            let packet = match packet_type {
                PacketType::ConnAck => MQTTPacket::ConnAck(crate::mqtt::mqttv4::connack::read(fixed_header, packet)?, None),
                PacketType::Publish => MQTTPacket::Publish(crate::mqtt::mqttv4::publish::read(fixed_header, packet)?, None),
                PacketType::PubAck => MQTTPacket::PubAck(crate::mqtt::mqttv4::puback::read(fixed_header, packet)?, None),
                PacketType::PubRec => MQTTPacket::PubRec(crate::mqtt::mqttv4::pubrec::read(fixed_header, packet)?, None),
                PacketType::PubRel => MQTTPacket::PubRel(crate::mqtt::mqttv4::pubrel::read(fixed_header, packet)?, None),
                PacketType::PubComp => MQTTPacket::PubComp(crate::mqtt::mqttv4::pubcomp::read(fixed_header, packet)?, None),
                PacketType::Subscribe => {
                    MQTTPacket::Subscribe(crate::mqtt::mqttv4::subscribe::read(fixed_header, packet)?, None)
                }
                PacketType::SubAck => MQTTPacket::SubAck(crate::mqtt::mqttv4::suback::read(fixed_header, packet)?, None),
                PacketType::Unsubscribe => {
                    MQTTPacket::Unsubscribe(crate::mqtt::mqttv4::unsubscribe::read(fixed_header, packet)?, None)
                }
                PacketType::UnsubAck => MQTTPacket::UnsubAck(crate::mqtt::mqttv4::unsuback::read(fixed_header, packet)?, None),
                PacketType::PingReq => MQTTPacket::PingReq(crate::mqtt::common::PingReq),
                PacketType::PingResp => MQTTPacket::PingResp(crate::mqtt::common::PingResp),
                // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
                // hit when Disconnect packet has properties which are only valid for MQTT V5
                PacketType::Disconnect => return Err(Error::InvalidProtocol),
                _ => unreachable!(),
            };
            return Ok(Some(packet));
        } else if protocol_version == 5{
            let packet = match packet_type {
                PacketType::ConnAck => {
                    let (conn_ack, conn_ack_properties) = crate::mqtt::mqttv5::connack::read(fixed_header, packet)?;
                    MQTTPacket::ConnAck(conn_ack, conn_ack_properties)
                }
                PacketType::Publish => {
                    let (publish, publish_properties) = crate::mqtt::mqttv5::publish::read(fixed_header, packet)?;
                    MQTTPacket::Publish(publish, publish_properties)
                }
                PacketType::PubAck => {
                    let (puback, puback_properties) = crate::mqtt::mqttv5::puback::read(fixed_header, packet)?;
                    MQTTPacket::PubAck(puback, puback_properties)
                }
                PacketType::PubRec => {
                    let (pubrec, pubrec_properties) = crate::mqtt::mqttv5::pubrec::read(fixed_header, packet)?;
                    MQTTPacket::PubRec(pubrec, pubrec_properties)
                }
                PacketType::PubRel => {
                    let (pubrel, pubrel_properteis) = crate::mqtt::mqttv5::pubrel::read(fixed_header, packet)?;
                    MQTTPacket::PubRel(pubrel, pubrel_properteis)
                }
                PacketType::PubComp => {
                    let (pubcomp, pubcomp_properties) = crate::mqtt::mqttv5::pubcomp::read(fixed_header, packet)?;
                    MQTTPacket::PubComp(pubcomp, pubcomp_properties)
                }
                PacketType::Subscribe => {
                    let (subscribe, subscribe_properties) = crate::mqtt::mqttv5::subscribe::read(fixed_header, packet)?;
                    MQTTPacket::Subscribe(subscribe, subscribe_properties)
                }
                PacketType::SubAck => {
                    let (suback, suback_properties) = crate::mqtt::mqttv5::suback::read(fixed_header, packet)?;
                    MQTTPacket::SubAck(suback, suback_properties)
                }
                PacketType::Unsubscribe => {
                    let (unsubscribe, unsubscribe_properties) =
                    crate::mqtt::mqttv5::unsubscribe::read(fixed_header, packet)?;
                    MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties)
                }
                PacketType::UnsubAck => {
                    let (unsuback, unsuback_properties) = crate::mqtt::mqttv5::unsuback::read(fixed_header, packet)?;
                    MQTTPacket::UnsubAck(unsuback, unsuback_properties)
                }
                PacketType::PingReq => MQTTPacket::PingReq(crate::mqtt::common::PingReq),
                PacketType::PingResp => MQTTPacket::PingResp(crate::mqtt::common::PingResp),
                // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
                // hit when Disconnect packet has properties which are only valid for MQTT V5
                PacketType::Disconnect => {
                    let (disconnect, disconnect_properties) =  crate::mqtt::mqttv5::disconnect::read(fixed_header, packet)?;
                    MQTTPacket::Disconnect(disconnect, disconnect_properties)
                }
                _ => unreachable!(),
            };
            return Ok(Some(packet));
        }

        return Err(Error::InvalidProtocol);
    }
}
