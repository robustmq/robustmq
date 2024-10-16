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

fn len(pubrel: &PubRel, properties: &Option<PubRelProperties>) -> usize {
    let mut len = 2 + 1; // pkid + reason

    // The reason code and property length can be omitted if the reason code is Success(0x00)
    // and there are no properties. In this case the remaining length of PubRel should be 2.
    //  <https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901144> can be referenced
    if pubrel.reason.unwrap() == PubRelReason::Success && properties.is_none() {
        return 2;
    }

    if let Some(p) = properties {
        let properties_len = properties::len(p);
        let properties_len_len = len_len(properties_len);
        len += properties_len_len + properties_len;
    } else {
        len += 1;
    }
    len
}

pub fn write(
    pubrel: &PubRel,
    properties: &Option<PubRelProperties>,
    buffer: &mut BytesMut,
) -> Result<usize, Error> {
    let len = len(pubrel, properties);
    buffer.put_u8(0x62);
    let count = write_remaining_length(buffer, len)?;
    buffer.put_u16(pubrel.pkid);

    // if there are no properties during success, reason code is optional as it is 0x00
    if pubrel.reason.unwrap() == PubRelReason::Success && properties.is_none() {
        return Ok(4);
    }

    buffer.put_u8(code(pubrel.reason.unwrap()));

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
) -> Result<(PubRel, Option<PubRelProperties>), Error> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    let pkid = read_u16(&mut bytes)?;
    if fixed_header.remaining_len == 2 {
        return Ok((
            PubRel {
                pkid,
                reason: Some(PubRelReason::Success),
            },
            None,
        ));
    }
    let ack_reason = read_u8(&mut bytes)?;
    if fixed_header.remaining_len < 4 {
        return Ok((
            PubRel {
                pkid,
                reason: Some(reason(ack_reason)?),
            },
            None,
        ));
    }

    let pubrel = PubRel {
        pkid,
        reason: Some(reason(ack_reason)?),
    };

    let properties = properties::read(&mut bytes)?;
    Ok((pubrel, properties))
}

mod properties {
    use super::*;

    pub fn len(properties: &PubRelProperties) -> usize {
        let mut len = 0;

        if let Some(reason) = &properties.reason_string {
            len += 1 + 2 + reason.len();
        }

        for (key, value) in properties.user_properties.iter() {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        len
    }

    pub fn write(properties: &PubRelProperties, buffer: &mut BytesMut) -> Result<(), Error> {
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

    pub fn read(bytes: &mut Bytes) -> Result<Option<PubRelProperties>, Error> {
        let mut reason_string = None;
        let mut user_properties = Vec::new();

        let (properties_len_len, properties_len) = length(bytes.iter())?;
        bytes.advance(properties_len_len);
        if properties_len == 0 {
            return Ok(None);
        }

        let mut cursor = 0;
        // read until cursor reaches the property length. It will skip this loop if properties_len is 0
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
                _ => return Err(Error::InvalidPacketType(prop)),
            }
        }
        Ok(Some(PubRelProperties {
            reason_string,
            user_properties,
        }))
    }
}

/// Connection return code type
fn reason(num: u8) -> Result<PubRelReason, Error> {
    let code = match num {
        0 => PubRelReason::Success,
        146 => PubRelReason::PacketIdentifierNotFound,
        num => return Err(Error::InvalidConnectReturnCode(num)),
    };

    Ok(code)
}

fn code(reason: PubRelReason) -> u8 {
    match reason {
        PubRelReason::Success => 0,
        PubRelReason::PacketIdentifierNotFound => 146,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pubrel_v5() {
        use super::*;

        let mut buffer = BytesMut::new();
        let pubrel = PubRel {
            pkid: 20u16,
            reason: Some(PubRelReason::Success),
        };

        let vec = vec![("username".to_string(), "Justin".to_string())];
        let properties = PubRelProperties {
            reason_string: Some(String::from("Success")),
            user_properties: vec,
        };

        // test the write function of PubRel in v5
        write(&pubrel, &Some(properties), &mut buffer).unwrap();

        // test the fixed_header part
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b0110_0010);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 33);

        // test the read function of pubrec packet and check the result of write function in MQTT v5
        let (x, y) = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(x.pkid, 20);
        assert_eq!(x.reason.unwrap(), PubRelReason::Success);

        let pubrel_properties = y.unwrap();
        assert_eq!(pubrel_properties.reason_string, Some("Success".to_string()));
        assert_eq!(
            pubrel_properties.user_properties.first(),
            Some(&("username".to_string(), "Justin".to_string()))
        );

        // test display of puback and puback_properties in v5
        println!("pubrel is {}", pubrel);
        println!("pubrel_properties is {}", pubrel_properties);
    }
}
