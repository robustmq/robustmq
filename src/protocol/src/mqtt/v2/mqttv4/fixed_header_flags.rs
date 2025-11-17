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

use crate::mqtt::v2::mqttv4::control_packet_type::ControlPacketType;
use crate::mqtt::v2::mqttv4::mqtt_protocol_error::MQTTProtocolError;
use crate::mqtt::v2::utils::radix::radix_handler;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum FixedHeaderFlags {
    Publish { dup: bool, qos: u8, retain: bool },
    Connect,
    ConnAck,
    PubAck,
    PubRec,
    PubRel,
    PubComp,
    Subscribe,
    SubAck,
    Unsubscribe,
    UnsubAck,
    PingReq,
    PingResp,
    Disconnect,
}
#[allow(dead_code)]
impl FixedHeaderFlags {
    pub(crate) fn parse(
        control_packet_type: ControlPacketType,
        binary_byte: u8,
    ) -> Result<Self, MQTTProtocolError> {
        Self::verify(control_packet_type.clone(), binary_byte)?;
        Self::create_factory(control_packet_type, binary_byte)
    }

    pub(super) fn verify(
        control_packet_type: ControlPacketType,
        binary_byte: u8,
    ) -> Result<(), MQTTProtocolError> {
        match control_packet_type {
            ControlPacketType::Connect
            | ControlPacketType::ConnAck
            | ControlPacketType::PubAck
            | ControlPacketType::PubRec
            | ControlPacketType::PubComp
            | ControlPacketType::UnsubAck
            | ControlPacketType::PingReq
            | ControlPacketType::PingResp
            | ControlPacketType::Disconnect
            | ControlPacketType::SubAck => {
                Ok(Self::check_reserved_value(binary_byte, 0b0000_0000)?)
            }
            ControlPacketType::PubRel
            | ControlPacketType::Subscribe
            | ControlPacketType::Unsubscribe => {
                Ok(Self::check_reserved_value(binary_byte, 0b0000_0010)?)
            }
            ControlPacketType::Publish => Ok(()),
        }
    }

    pub(super) fn check_reserved_value(
        binary_byte: u8,
        reserved_value: u8,
    ) -> Result<(), MQTTProtocolError> {
        if radix_handler::low_nibble(binary_byte) != reserved_value {
            return Err(MQTTProtocolError::InvalidFixedHeaderFlags);
        }
        Ok(())
    }

    pub(self) fn create_factory(
        control_packet_type: ControlPacketType,
        binary_byte: u8,
    ) -> Result<Self, MQTTProtocolError> {
        match control_packet_type {
            ControlPacketType::Publish => Self::create_publish_fixed_header_flags(binary_byte),
            ControlPacketType::Connect => Ok(FixedHeaderFlags::Connect),
            ControlPacketType::ConnAck => Ok(FixedHeaderFlags::ConnAck),
            ControlPacketType::PubAck => Ok(FixedHeaderFlags::PubAck),
            ControlPacketType::PubRec => Ok(FixedHeaderFlags::PubRec),
            ControlPacketType::PubRel => Ok(FixedHeaderFlags::PubRel),
            ControlPacketType::PubComp => Ok(FixedHeaderFlags::PubComp),
            ControlPacketType::Subscribe => Ok(FixedHeaderFlags::Subscribe),
            ControlPacketType::SubAck => Ok(FixedHeaderFlags::SubAck),
            ControlPacketType::Unsubscribe => Ok(FixedHeaderFlags::Unsubscribe),
            ControlPacketType::UnsubAck => Ok(FixedHeaderFlags::UnsubAck),
            ControlPacketType::PingReq => Ok(FixedHeaderFlags::PingReq),
            ControlPacketType::PingResp => Ok(FixedHeaderFlags::PingResp),
            ControlPacketType::Disconnect => Ok(FixedHeaderFlags::Disconnect),
        }
    }

    pub(self) fn create_publish_fixed_header_flags(
        binary_byte: u8,
    ) -> Result<Self, MQTTProtocolError> {
        let low4bits = radix_handler::low_nibble(binary_byte);
        let dup = (low4bits & 0b0000_1000) >> 3 == 1;
        let qos = (low4bits & 0b0000_0110) >> 1;
        if qos > 2 {
            return Err(MQTTProtocolError::QoSLevelNotSupported(qos));
        }
        let retain = (low4bits & 0b0000_0001) == 1;
        Ok(FixedHeaderFlags::Publish { dup, qos, retain })
    }
}

#[cfg(test)]
mod fixed_header_flags_tests {
    use crate::mqtt::v2::mqttv4::control_packet_type::ControlPacketType;
    use crate::mqtt::v2::mqttv4::fixed_header_flags::FixedHeaderFlags;
    use crate::mqtt::v2::mqttv4::mqtt_protocol_error::MQTTProtocolError;

    #[test]
    fn fixed_header_connect_reserved_flags_should_be_0000() {
        let byte = 0b0001_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Connect);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok())
    }
    #[test]
    fn fixed_header_connack_reserved_flags_should_be_0000() {
        let byte = 0b0010_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::ConnAck);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_publish_reserved_flags_should_be_0000_to_1111() {
        let byte = 0b0011_0110;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Publish);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_puback_reserved_flags_should_be_0000() {
        let byte = 0b0100_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubAck);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_pubrec_reserved_flags_should_be_0000() {
        let byte = 0b0101_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubRec);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_pubrel_reserved_flags_should_be_0000() {
        let byte = 0b0110_0010;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubRel);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_pubcomp_reserved_flags_should_be_0000() {
        let byte = 0b0111_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubComp);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_subscribe_reserved_flags_should_be_0010() {
        let byte = 0b1000_0010;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Subscribe);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_suback_reserved_flags_should_be_0000() {
        let byte = 0b1001_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::SubAck);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_unsubscribe_reserved_flags_should_be_0010() {
        let byte = 0b1010_0010;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Unsubscribe);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_unsuback_reserved_flags_should_be_0000() {
        let byte = 0b1011_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::UnsubAck);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_pingreq_reserved_flags_should_be_0000() {
        let byte = 0b1100_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PingReq);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_pingresp_reserved_flags_should_be_0000() {
        let byte = 0b1101_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PingResp);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }
    #[test]
    fn fixed_header_disconnect_reserved_flags_should_be_0000() {
        let byte = 0b1110_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Disconnect);
        assert!(FixedHeaderFlags::verify(packet_type, byte).is_ok());
    }

    #[test]
    fn fixed_header_invalid_reserved_flags_should_error() {
        let byte = 0b1110_0010;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Disconnect);
        let result = FixedHeaderFlags::verify(packet_type, byte);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(MQTTProtocolError::InvalidFixedHeaderFlags)
        ))
    }

    #[test]
    fn fixed_header_publish_extract_reserved_flags_should_get_dup_value() {
        let byte = 0b0011_1000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Publish);
        let publish_flags = FixedHeaderFlags::parse(packet_type, byte).unwrap();
        if let FixedHeaderFlags::Publish {
            dup,
            qos: _,
            retain: _,
        } = publish_flags
        {
            assert!(dup);
        }
    }

    #[test]
    fn fixed_header_publish_extract_reserved_flags_should_get_retain_value() {
        let byte = 0b0011_0001;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Publish);
        let publish_flags = FixedHeaderFlags::parse(packet_type, byte).unwrap();
        if let FixedHeaderFlags::Publish {
            dup: _,
            qos: _,
            retain,
        } = publish_flags
        {
            assert!(retain);
        }
    }
    #[test]
    fn fixed_header_publish_extract_reserved_flags_should_get_qos_value() {
        let byte = 0b0011_0100;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Publish);
        let publish_flags = FixedHeaderFlags::parse(packet_type, byte).unwrap();
        if let FixedHeaderFlags::Publish {
            dup: _,
            qos,
            retain: _,
        } = publish_flags
        {
            assert_eq!(qos, 2);
        }
    }
    #[test]
    fn fixed_header_publish_extract_reserved_flags_qos_invalid_should_error() {
        let byte = 0b0011_0110;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Publish);
        let result = FixedHeaderFlags::parse(packet_type, byte);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(MQTTProtocolError::QoSLevelNotSupported(3))
        ))
    }
}
