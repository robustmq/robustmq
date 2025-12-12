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

use super::*;
use common_base::error::mqtt_protocol_error::MQTTProtocolError;

fn len(puback: &PubAck, properties: &Option<PubAckProperties>) -> usize {
    let mut len = 2 + 1; // pkid + reason code

    // If there are no properties, sending reason code is optional
    if let Some(reason) = puback.reason {
        if reason == PubAckReason::Success && properties.is_none() {
            return 2;
        }
    }

    if let Some(p) = properties {
        let properties_len = properties::len(p);
        let properties_len_len = len_len(properties_len);
        len += properties_len_len + properties_len;
    } else {
        // just 1 byte representing 0 len properties
        len += 1;
    }

    len
}

pub fn write(
    puback: &PubAck,
    properties: &Option<PubAckProperties>,
    buffer: &mut BytesMut,
) -> Result<usize, MQTTProtocolError> {
    let len = len(puback, properties);
    buffer.put_u8(0x40); // 1st byte
    let count = write_remaining_length(buffer, len)?;
    buffer.put_u16(puback.pkid);
    // Reason code is optional with success if there are no properties
    if let Some(reason) = puback.reason {
        if reason == PubAckReason::Success && properties.is_none() {
            return Ok(4);
        }
        buffer.put_u8(code(reason));
    }
    if let Some(p) = properties {
        properties::write(p, buffer)?;
    } else {
        write_remaining_length(buffer, 0)?;
    }

    Ok(1 + count + len)
}

pub fn read(
    fixed_header: FixedHeader,
    mut bytes: Bytes,
) -> Result<(PubAck, Option<PubAckProperties>), MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);
    let pkid = read_u16(&mut bytes)?;

    // No reason code or properties if remaining length is 2
    if fixed_header.remaining_len == 2 {
        return Ok((
            PubAck {
                pkid,
                reason: Some(PubAckReason::Success),
            },
            None,
        ));
    }
    // No properties len or properties if remaining len > 2 but < 4
    let ack_reason = read_u8(&mut bytes)?;
    if fixed_header.remaining_len < 4 {
        return Ok((
            PubAck {
                pkid,
                reason: Some(reason(ack_reason)?),
            },
            None,
        ));
    }

    let puback = PubAck {
        pkid,
        reason: Some(reason(ack_reason)?),
    };

    let properties = properties::read(&mut bytes)?;
    Ok((puback, properties))
}

mod properties {
    use super::*;

    pub fn len(properties: &PubAckProperties) -> usize {
        let mut len = 0;
        if let Some(reason) = &properties.reason_string {
            len += 1 + 2 + reason.len();
        }
        for (key, value) in properties.user_properties.iter() {
            len += 1 + 2 + key.len() + 2 + value.len();
        }
        len
    }

    pub fn write(
        properties: &PubAckProperties,
        buffer: &mut BytesMut,
    ) -> Result<(), MQTTProtocolError> {
        let len = len(properties);
        write_remaining_length(buffer, len)?;

        if let Some(reason) = &properties.reason_string {
            buffer.put_u8(PropertyType::ReasonString as u8);
            write_mqtt_string(buffer, reason);
        }

        for (key, value) in properties.user_properties.iter() {
            buffer.put_u8(PropertyType::UserProperty as u8);
            write_mqtt_string(buffer, key);
            write_mqtt_string(buffer, value);
        }

        Ok(())
    }

    pub fn read(bytes: &mut Bytes) -> Result<Option<PubAckProperties>, MQTTProtocolError> {
        let mut reason_string = None;
        let mut user_properties = Vec::new();

        let (properties_len_len, properties_len) = length(bytes.iter())?;
        bytes.advance(properties_len_len);
        if properties_len == 0 {
            return Ok(None);
        }
        let mut cursor = 0;
        // read until cursor reaches property length. properties_len = 0 will skip this loop
        while cursor < properties_len {
            let prop = read_u8(bytes)?;
            cursor += 1;

            match property(prop)? {
                PropertyType::ReasonString => {
                    let reason = read_mqtt_string(bytes)?;
                    cursor += 2 + reason.len();
                    reason_string = Some(reason);
                }

                PropertyType::UserProperty => {
                    let key = read_mqtt_string(bytes)?;
                    let value = read_mqtt_string(bytes)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                _ => return Err(MQTTProtocolError::InvalidPropertyType(prop)),
            }
        }
        Ok(Some(PubAckProperties {
            reason_string,
            user_properties,
        }))
    }
}

/// Connection return code type
fn reason(num: u8) -> Result<PubAckReason, MQTTProtocolError> {
    let code = match num {
        0 => PubAckReason::Success,
        16 => PubAckReason::NoMatchingSubscribers,
        128 => PubAckReason::UnspecifiedError,
        131 => PubAckReason::ImplementationSpecificError,
        135 => PubAckReason::NotAuthorized,
        144 => PubAckReason::TopicNameInvalid,
        145 => PubAckReason::PacketIdentifierInUse,
        151 => PubAckReason::QuotaExceeded,
        153 => PubAckReason::PayloadFormatInvalid,
        num => return Err(MQTTProtocolError::InvalidConnectReturnCode(num)),
    };
    Ok(code)
}

fn code(reason: PubAckReason) -> u8 {
    match reason {
        PubAckReason::Success => 0,
        PubAckReason::NoMatchingSubscribers => 16,
        PubAckReason::UnspecifiedError => 128,
        PubAckReason::ImplementationSpecificError => 131,
        PubAckReason::NotAuthorized => 135,
        PubAckReason::TopicNameInvalid => 144,
        PubAckReason::PacketIdentifierInUse => 145,
        PubAckReason::QuotaExceeded => 151,
        PubAckReason::PayloadFormatInvalid => 153,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_puback_v5() {
        use super::*;

        let mut buffer = BytesMut::new();
        let puback = PubAck {
            pkid: 20u16,
            reason: Some(PubAckReason::NotAuthorized),
        };

        let vec = vec![("username".to_string(), "Justin".to_string())];
        let properties = PubAckProperties {
            reason_string: Some(String::from("user authorization failed")),
            user_properties: vec,
        };

        // test the write function of PubAck in v5
        write(&puback.clone(), &Some(properties.clone()), &mut buffer).unwrap();

        // test the fixed_header part
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b0100_0000);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 51);

        // test the read function of puback packet and check the result of write function in MQTT v5
        let (pub_ack_read, option_pub_ack_properties) =
            read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(pub_ack_read.pkid, 20u16);
        assert_eq!(pub_ack_read.reason.unwrap(), PubAckReason::NotAuthorized);

        let puback_properties = option_pub_ack_properties.clone().unwrap();
        assert_eq!(
            puback_properties.reason_string,
            Some("user authorization failed".to_string())
        );
        assert_eq!(
            puback_properties.user_properties.first(),
            Some(&("username".to_string(), "Justin".to_string()))
        );

        // test display of puback and puback_properties in v5
        assert_eq!(puback.to_string(), pub_ack_read.to_string());
        assert_eq!(properties.to_string(), puback_properties.to_string());
    }
}
