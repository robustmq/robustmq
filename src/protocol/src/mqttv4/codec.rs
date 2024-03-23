use bytes::BytesMut;
use tokio_util::codec;

use super::{
    check, connack, connect, disconnect, ping, puback, pubcomp, publish, pubrec, pubrel, suback,
    subscribe, unsuback, unsubscribe, Error, Packet, PacketType,
};

#[derive(Clone)]
pub struct Mqtt4Codec {}

impl Mqtt4Codec {
    pub fn new() -> Mqtt4Codec {
        return Mqtt4Codec {};
    }
}

impl codec::Encoder<Packet> for Mqtt4Codec {
    type Error = super::Error;
    fn encode(&mut self, packet: Packet, buffer: &mut BytesMut) -> Result<(), Self::Error> {
        match packet {
            Packet::Connect(connect, None, last_will, None, login) => {
                connect::write(&connect, &login, &last_will, buffer)?
            }
            Packet::ConnAck(connack, _) => connack::write(&connack, buffer)?,
            Packet::Publish(publish, None) => publish::write(&publish, buffer)?,
            Packet::PubAck(puback, None) => puback::write(&puback, buffer)?,
            Packet::PubRec(pubrec, None) => pubrec::write(&pubrec, buffer)?,
            Packet::PubRel(pubrel, None) => pubrel::write(&pubrel, buffer)?,
            Packet::PubComp(pubcomp, None) => pubcomp::write(&pubcomp, buffer)?,
            Packet::Subscribe(subscribe, None) => subscribe::write(&subscribe, buffer)?,
            Packet::SubAck(suback, None) => suback::write(&suback, buffer)?,
            Packet::Unsubscribe(unsubscribe, None) => unsubscribe::write(&unsubscribe, buffer)?,
            Packet::UnsubAck(unsuback, None) => unsuback::write(&unsuback, buffer)?,
            Packet::PingReq(pingreq) => ping::pingreq::write(buffer)?,
            Packet::PingResp(pingresp) => ping::pingresp::write(buffer)?,
            Packet::Disconnect(disconnect, None) => disconnect::write(&disconnect, buffer)?,

            //Packet::
            _=> unreachable!(
                "This branch only matches for packets with Properties, which is not possible in MQTT V4",
            ),
        };
        Ok(())
    }
}

impl codec::Decoder for Mqtt4Codec {
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
                let (connect, login, lastwill) = connect::read(fixed_header, packet)?;
                Packet::Connect(connect, None, lastwill, None, login)
            }
            PacketType::ConnAck => Packet::ConnAck(connack::read(fixed_header, packet)?, None),
            PacketType::Publish => Packet::Publish(publish::read(fixed_header, packet)?, None),
            PacketType::PubAck => Packet::PubAck(puback::read(fixed_header, packet)?, None),
            PacketType::PubRec => Packet::PubRec(pubrec::read(fixed_header, packet)?, None),
            PacketType::PubRel => Packet::PubRel(pubrel::read(fixed_header, packet)?, None),
            PacketType::PubComp => Packet::PubComp(pubcomp::read(fixed_header, packet)?, None),
            PacketType::Subscribe => {
                Packet::Subscribe(subscribe::read(fixed_header, packet)?, None)
            }
            PacketType::SubAck => Packet::SubAck(suback::read(fixed_header, packet)?, None),
            PacketType::Unsubscribe => {
                Packet::Unsubscribe(unsubscribe::read(fixed_header, packet)?, None)
            }
            PacketType::UnsubAck => Packet::UnsubAck(unsuback::read(fixed_header, packet)?, None),
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
