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

use crate::mqtt::v2::byte_adapter::byte_operations::ByteOperations;
use crate::mqtt::v2::mqttv4::control_packet_type::ControlPacketType;
use crate::mqtt::v2::mqttv4::fixed_header_flags::FixedHeaderFlags;
use crate::mqtt::v2::mqttv4::mqtt_protocol_error::MQTTProtocolError;
use crate::mqtt::v2::mqttv4::remaining_length::remaining_length_parser;

#[allow(dead_code)]
pub struct FixedHeader {
    control_packet_type: ControlPacketType,
    fixed_header_reserve_flags: FixedHeaderFlags,
    remaining_length: u32,
}

#[allow(dead_code)]
impl FixedHeader {
    pub(crate) fn parse(bytes: &mut impl ByteOperations) -> Result<FixedHeader, MQTTProtocolError> {
        let first_byte = bytes
            .read_a_byte()
            .ok_or(MQTTProtocolError::PacketTooShort)?;
        let control_packet_type = ControlPacketType::parse(first_byte)?;
        let fixed_header_reserve_flags =
            FixedHeaderFlags::parse(control_packet_type.clone(), first_byte)?;

        let remaining_length = remaining_length_parser::parse(bytes)?;

        Ok(FixedHeader {
            control_packet_type,
            fixed_header_reserve_flags,
            remaining_length,
        })
    }
}

#[cfg(test)]
mod fixed_header_tests {
    use crate::mqtt::v2::byte_adapter::byte_operations::ByteOperations;
    use crate::mqtt::v2::mqttv4::control_packet_type::ControlPacketType;
    use crate::mqtt::v2::mqttv4::fixed_header::FixedHeader;
    use crate::mqtt::v2::mqttv4::fixed_header_flags::FixedHeaderFlags;
    use crate::mqtt::v2::mqttv4::mqtt_protocol_error::MQTTProtocolError;
    use bytes::BytesMut;

    #[test]
    fn fixed_header_can_parse_connect_packet() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_bytes(&[0b0001_0000, 0b0000_0010, 0b0000_0100]);
        let fixed_header = FixedHeader::parse(&mut bytes_mut).unwrap();
        assert_eq!(fixed_header.control_packet_type, ControlPacketType::Connect);
        assert_eq!(
            fixed_header.fixed_header_reserve_flags,
            FixedHeaderFlags::Connect
        );
        assert_eq!(fixed_header.remaining_length, 2);
        assert_eq!(bytes_mut.read_a_byte().unwrap(), 0b0000_0100);
    }

    #[test]
    fn fixed_header_can_parse_publish_packet() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_bytes(&[0b0011_1101, 0b0000_0011, 0b0000_0101, 0b0000_0110]);
        let fixed_header = FixedHeader::parse(&mut bytes_mut).unwrap();
        assert_eq!(fixed_header.control_packet_type, ControlPacketType::Publish);
        assert_eq!(
            fixed_header.fixed_header_reserve_flags,
            FixedHeaderFlags::Publish {
                dup: true,
                qos: 2,
                retain: true
            }
        );
        assert_eq!(fixed_header.remaining_length, 3);
        assert_eq!(bytes_mut.read_a_byte().unwrap(), 0b0000_0101);
    }

    #[test]
    fn fixe_header_parse_fails_on_short_packet() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_bytes(&[0b0001_0000]);
        let result = FixedHeader::parse(&mut bytes_mut);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            MQTTProtocolError::PacketTooShort
        ));
    }
}
