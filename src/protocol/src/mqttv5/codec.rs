use bytes::BytesMut;
use tokio_util::codec;

use crate::mqtt::LastWillProperties;

use super::{
    check, connack, connect, disconnect, ping, puback, pubcomp, publish, pubrec, pubrel, suback,
    subscribe, unsuback, unsubscribe, Error, Packet, PacketType,
};

pub struct Mqtt5Codec {}

impl Mqtt5Codec {
    pub fn new() -> Mqtt5Codec {
        return Mqtt5Codec {};
    }
}

impl codec::Encoder<Packet> for Mqtt5Codec {
    type Error = super::Error;
    fn encode(&mut self, packet: Packet, buffer: &mut BytesMut) -> Result<(), Self::Error> {
        let size = match packet {
            Packet::Connect(connect, properties, last_will, last_will_peoperties, login) => {
                connect::write(&connect, &properties, &last_will,  &last_will_peoperties, &login, buffer)?
            }
            Packet::ConnAck(connack, conn_ack_properties) => connack::write(&connack,&conn_ack_properties, buffer)?,
            Packet::Publish(publish, publish_properties ) => publish::write(&publish, &publish_properties,buffer)?,
            Packet::PubAck(puback, pub_ack_properties) => puback::write(&puback, &pub_ack_properties,buffer)?,
            Packet::PubRec(pubrec, pub_rec_properties) => pubrec::write(&pubrec, &pub_rec_properties,buffer)?,
            Packet::PubRel(pubrel, pub_rel_properties) => pubrel::write(&pubrel, &pub_rel_properties,buffer)?,
            Packet::PubComp(pubcomp, pub_comp_properties) => pubcomp::write(&pubcomp, &pub_comp_properties,buffer)?,
            Packet::Subscribe(subscribe, subscribe_properties) => subscribe::write(&subscribe,&subscribe_properties, buffer)?,
            Packet::SubAck(suback, suback_properties) => suback::write(&suback, &suback_properties,buffer)?,
            Packet::Unsubscribe(unsubscribe, unsubscribe_properties) => unsubscribe::write(&unsubscribe, &unsubscribe_properties,buffer)?,
            Packet::UnsubAck(unsuback, unsuback_properties) => unsuback::write(&unsuback, &unsuback_properties,buffer)?,
            Packet::PingReq(pingreq) => ping::pingreq::write(buffer)?,
            Packet::PingResp(pingresp) => ping::pingresp::write(buffer)?,
            Packet::Disconnect(disconnect, disconnect_properties) => disconnect::write(&disconnect, &disconnect_properties,buffer)?,

            //Packet::
            _=> unreachable!(
                "This branch only matches for packets with Properties, which is not possible in MQTT V4",
            ),
        };
        Ok(())
    }
}

impl codec::Decoder for Mqtt5Codec {
    type Item = Packet;
    type Error = super::Error;
    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let fixed_header = check(stream.iter(), 1000000)?;
        // Test with a stream with exactly the size to check border panics
        let packet = stream.split_to(fixed_header.frame_length());
        let packet_type = fixed_header.packet_type()?;
        let packet = packet.freeze();
        let packet = match packet_type {
            PacketType::Connect => {
                let (connect, properties, last_will, last_will_properties, login) =
                    connect::read(fixed_header, packet)?;
                Packet::Connect(connect, properties, last_will, last_will_properties, login)
            }
            PacketType::ConnAck => {
                let (conn_ack, conn_ack_properties) = connack::read(fixed_header, packet)?;
                Packet::ConnAck(conn_ack, conn_ack_properties)
            }
            PacketType::Publish => {
                let (publish, publish_properties) = publish::read(fixed_header, packet)?;
                Packet::Publish(publish, publish_properties)
            }
            PacketType::PubAck => {
                let (puback, puback_properties) = puback::read(fixed_header, packet)?;
                Packet::PubAck(puback, puback_properties)
            }
            PacketType::PubRec => {
                let (pubrec, pubrec_properties) = pubrec::read(fixed_header, packet)?;
                Packet::PubRec(pubrec, pubrec_properties)
            }
            PacketType::PubRel => {
                let (pubrel, pubrel_properteis) = pubrel::read(fixed_header, packet)?;
                Packet::PubRel(pubrel, pubrel_properteis)
            }
            PacketType::PubComp => {
                let (pubcomp, pubcomp_properties) = pubcomp::read(fixed_header, packet)?;
                Packet::PubComp(pubcomp, pubcomp_properties)
            }
            PacketType::Subscribe => {
                let (subscribe, subscribe_properties) = subscribe::read(fixed_header, packet)?;
                Packet::Subscribe(subscribe, subscribe_properties)
            }
            PacketType::SubAck => {
                let (suback, suback_properties) = suback::read(fixed_header, packet)?;
                Packet::SubAck(suback, suback_properties)
            },
            PacketType::Unsubscribe => {
                let (unsubscribe, unsubscribe_properties) = unsubscribe::read(fixed_header, packet)?;
                Packet::Unsubscribe(unsubscribe, unsubscribe_properties)
            }
            PacketType::UnsubAck => {
                let (unsuback, unsuback_properties) = unsuback::read(fixed_header, packet)?;
                Packet::UnsubAck(unsuback, unsuback_properties)
            },
            PacketType::PingReq => Packet::PingReq(super::PingReq),
            PacketType::PingResp => Packet::PingResp(super::PingResp),
            // MQTT V4 Disconnect packet gets handled in the previous check, this branch gets
            // hit when Disconnect packet has properties which are only valid for MQTT V5
            PacketType::Disconnect => return Err(Error::InvalidProtocol),
            _ => unreachable!(),
        };
        return Ok(Some(packet));
    }
}
