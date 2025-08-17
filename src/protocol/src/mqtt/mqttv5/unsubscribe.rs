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

pub fn len(unsubscribe: &Unsubscribe, properties: &Option<UnsubscribeProperties>) -> usize {
    // Packet id + length of filters (unlike subscribe, this just a string.
    // Hence 2 is prefixed for len per filter)
    let mut len = 2 + unsubscribe.filters.iter().fold(0, |s, t| 2 + s + t.len());

    if let Some(p) = properties {
        let properties_len = properties::len(p);
        let properties_len_len = len_len(properties_len);
        len += properties_len_len + properties_len;
    } else {
        // just 1 byte representing 0 length
        len += 1;
    }

    len
}

pub fn write(
    unsubscribe: &Unsubscribe,
    properties: &Option<UnsubscribeProperties>,
    buffer: &mut BytesMut,
) -> Result<usize, MQTTProtocolError> {
    buffer.put_u8(0xA2);
    // write remaining length
    let remaining_len = len(unsubscribe, properties);
    let remaining_len_bytes = write_remaining_length(buffer, remaining_len)?;

    // write packet id
    buffer.put_u16(unsubscribe.pkid);

    if let Some(p) = properties {
        properties::write(p, buffer)?;
    } else {
        write_remaining_length(buffer, 0)?;
    }
    // write filters
    for filter in unsubscribe.filters.iter() {
        write_mqtt_string(buffer, filter);
    }

    Ok(1 + remaining_len_bytes + remaining_len)
}

pub fn read(
    fixed_header: FixedHeader,
    mut bytes: Bytes,
) -> Result<(Unsubscribe, Option<UnsubscribeProperties>), MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    let pkid = read_u16(&mut bytes)?;
    let properties = properties::read(&mut bytes)?;

    let mut filters = Vec::with_capacity(1);
    while bytes.has_remaining() {
        let filter = read_mqtt_string(&mut bytes)?;
        filters.push(filter);
    }

    let unsubscribe = Unsubscribe { pkid, filters };
    Ok((unsubscribe, properties))
}

mod properties {
    use super::*;

    pub fn len(properties: &UnsubscribeProperties) -> usize {
        let mut len = 0;

        for (key, value) in properties.user_properties.iter() {
            len += 1 + 2 + key.len() + 2 + value.len();
        }
        len
    }

    pub fn read(bytes: &mut Bytes) -> Result<Option<UnsubscribeProperties>, MQTTProtocolError> {
        let mut user_properties = Vec::new();

        let (properties_len_len, properties_len) = length(bytes.iter())?;
        bytes.advance(properties_len_len);

        if properties_len == 0 {
            return Ok(None);
        }

        let mut cursor = 0;
        // read until cursor reaches property length. it will skip this loop if properties_len is 0
        while cursor < properties_len {
            let prop = read_u8(bytes)?;
            cursor += 1;

            match property(prop)? {
                PropertyType::UserProperty => {
                    let key = read_mqtt_string(bytes)?;
                    let value = read_mqtt_string(bytes)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                _ => return Err(MQTTProtocolError::InvalidPacketType(prop)),
            }
        }
        Ok(Some(UnsubscribeProperties { user_properties }))
    }

    pub fn write(
        properties: &UnsubscribeProperties,
        buffer: &mut BytesMut,
    ) -> Result<(), MQTTProtocolError> {
        let len = len(properties);
        write_remaining_length(buffer, len)?;

        for (key, value) in properties.user_properties.iter() {
            buffer.put_u8(PropertyType::UserProperty as u8);
            write_mqtt_string(buffer, key);
            write_mqtt_string(buffer, value);
        }
        Ok(())
    }
}
