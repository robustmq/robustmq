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

use crate::mqtt::v2::mqttv4::mqtt_protocol_error::MQTTProtocolError;
use crate::mqtt::v2::utils::radix::radix_handler;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ControlPacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
    PubAck = 4,
    PubRec = 5,
    PubRel = 6,
    PubComp = 7,
    Subscribe = 8,
    SubAck = 9,
    Unsubscribe = 10,
    UnsubAck = 11,
    PingReq = 12,
    PingResp = 13,
    Disconnect = 14,
}
#[allow(dead_code)]
impl ControlPacketType {
    pub(crate) fn parse(binary_byte: u8) -> Result<ControlPacketType, MQTTProtocolError> {
        let high4bits_to8bits = radix_handler::high_nibble(binary_byte);
        let value = radix_handler::binary_byte_to_decimal(high4bits_to8bits);
        match value {
            1 => Ok(ControlPacketType::Connect),
            2 => Ok(ControlPacketType::ConnAck),
            3 => Ok(ControlPacketType::Publish),
            4 => Ok(ControlPacketType::PubAck),
            5 => Ok(ControlPacketType::PubRec),
            6 => Ok(ControlPacketType::PubRel),
            7 => Ok(ControlPacketType::PubComp),
            8 => Ok(ControlPacketType::Subscribe),
            9 => Ok(ControlPacketType::SubAck),
            10 => Ok(ControlPacketType::Unsubscribe),
            11 => Ok(ControlPacketType::UnsubAck),
            12 => Ok(ControlPacketType::PingReq),
            13 => Ok(ControlPacketType::PingResp),
            14 => Ok(ControlPacketType::Disconnect),
            _ => Err(MQTTProtocolError::InvalidPacketType),
        }
    }
}

#[cfg(test)]
mod control_packet_type_tests {
    use crate::mqtt::v2::mqttv4::control_packet_type::ControlPacketType;
    use crate::mqtt::v2::mqttv4::mqtt_protocol_error::MQTTProtocolError;
    #[test]
    fn control_packet_type_parse_connect() {
        let byte = 0b0001_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Connect);
    }

    #[test]
    fn control_packet_type_parse_connack() {
        let byte = 0b0010_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::ConnAck);
    }

    #[test]
    fn control_packet_type_parse_publish() {
        let byte = 0b0011_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Publish);
    }

    #[test]
    fn control_packet_type_parse_puback() {
        let byte = 0b0100_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubAck);
    }

    #[test]
    fn control_packet_type_parse_pubrec() {
        let byte = 0b0101_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubRec);
    }

    #[test]
    fn control_packet_type_parse_pubrel() {
        let byte = 0b0110_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubRel);
    }

    #[test]
    fn control_packet_type_parse_pubcomp() {
        let byte = 0b0111_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PubComp);
    }

    #[test]
    fn control_packet_type_parse_subscribe() {
        let byte = 0b1000_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Subscribe);
    }

    #[test]
    fn control_packet_type_parse_suback() {
        let byte = 0b1001_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::SubAck);
    }

    #[test]
    fn control_packet_type_parse_unsubscribe() {
        let byte = 0b1010_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Unsubscribe);
    }

    #[test]
    fn control_packet_type_parse_unsuback() {
        let byte = 0b1011_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::UnsubAck);
    }

    #[test]
    fn control_packet_type_parse_pingreq() {
        let byte = 0b1100_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PingReq);
    }

    #[test]
    fn control_packet_type_parse_pingresp() {
        let byte = 0b1101_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::PingResp);
    }

    #[test]
    fn control_packet_type_parse_disconnect() {
        let byte = 0b1110_0000;
        let packet_type = ControlPacketType::parse(byte).unwrap();
        assert_eq!(packet_type, ControlPacketType::Disconnect);
    }

    #[test]
    fn control_packet_type_parse_invalid() {
        let byte = 0b1111_0000;
        let result = ControlPacketType::parse(byte);
        assert!(result.is_err());
        assert!(matches!(result, Err(MQTTProtocolError::InvalidPacketType)));

        let byte = 0b0000_0000;
        let result = ControlPacketType::parse(byte);
        assert!(result.is_err());
        assert!(matches!(result, Err(MQTTProtocolError::InvalidPacketType)));
    }
}
