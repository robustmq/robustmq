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

pub fn len(subscribe: &Subscribe, properties: &Option<SubscribeProperties>) -> usize {
    let mut len = 2 + subscribe.filters.iter().fold(0, |s, t| s + filter::len(t));

    if let Some(p) = properties {
        let properties_len = properties::len(p);
        let properties_len_len = len_len(properties_len);
        len += properties_len_len + properties_len;
    } else {
        // just 1 byte representing 0 len
        len += 1;
    }

    len
}

pub fn read(
    fixed_header: FixedHeader,
    mut bytes: Bytes,
) -> Result<(Subscribe, Option<SubscribeProperties>), MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    let pkid = read_u16(&mut bytes)?;
    let properties = properties::read(&mut bytes)?;
    // variable header size = 2 (packet identifier)
    let mut filters = Vec::new();
    while bytes.has_remaining() {
        let path = read_mqtt_string(&mut bytes)?;
        let options = read_u8(&mut bytes)?;
        let requested_qos = options & 0b0000_0011;

        let nolocal = (options >> 2) & 0b0000_0001;
        let nolocal = nolocal != 0;

        let preserve_retain = (options >> 3) & 0b0000_0001;
        let preserve_retain = preserve_retain != 0;

        let retain_forward_rule = (options >> 4) & 0b0000_0011;
        let retain_forward_rule = match retain_forward_rule {
            0 => RetainHandling::OnEverySubscribe,
            1 => RetainHandling::OnNewSubscribe,
            2 => RetainHandling::Never,
            r => return Err(MQTTProtocolError::InvalidRetainForwardRule(r)),
        };

        filters.push(Filter {
            path,
            qos: qos(requested_qos).ok_or(MQTTProtocolError::InvalidQoS(requested_qos))?,
            no_local: nolocal,
            preserve_retain,
            retain_handling: retain_forward_rule,
        });
    }

    match filters.len() {
        0 => Err(MQTTProtocolError::EmptySubscription),
        _ => Ok((
            Subscribe {
                packet_identifier: pkid,
                filters,
            },
            properties,
        )),
    }
}

pub fn write(
    subscribe: &Subscribe,
    properties: &Option<SubscribeProperties>,
    buffer: &mut BytesMut,
) -> Result<usize, MQTTProtocolError> {
    // write packet type
    buffer.put_u8(0x82);

    // write remaining length
    let remaining_len = len(subscribe, properties);
    let remaining_len_bytes = write_remaining_length(buffer, remaining_len)?;

    // write packet id
    buffer.put_u16(subscribe.packet_identifier);

    if let Some(p) = properties {
        properties::write(p, buffer)?;
    } else {
        write_remaining_length(buffer, 0)?;
    }

    // write filters
    for f in subscribe.filters.iter() {
        filter::write(f, buffer);
    }

    Ok(1 + remaining_len_bytes + remaining_len)
}

mod filter {
    use super::*;

    pub fn len(filter: &Filter) -> usize {
        // filter len + filter + options
        2 + filter.path.len() + 1
    }

    pub fn write(filter: &Filter, buffer: &mut BytesMut) {
        let mut options = 0;
        options |= filter.qos as u8;

        if filter.no_local {
            options |= 0b0000_0100;
        }

        if filter.preserve_retain {
            options |= 0b0000_1000;
        }

        options |= match filter.retain_handling {
            RetainHandling::OnEverySubscribe => 0b0000_0000,
            RetainHandling::OnNewSubscribe => 0b0001_0000,
            RetainHandling::Never => 0b0010_0000,
        };

        write_mqtt_string(buffer, filter.path.as_str());
        buffer.put_u8(options);
    }
}

mod properties {
    use super::*;

    pub fn len(properties: &SubscribeProperties) -> usize {
        let mut len = 0;

        if let Some(id) = &properties.subscription_identifier {
            len += 1 + len_len(*id);
        }

        for (key, value) in properties.user_properties.iter() {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        len
    }

    pub fn read(bytes: &mut Bytes) -> Result<Option<SubscribeProperties>, MQTTProtocolError> {
        let mut id = None;
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
                PropertyType::SubscriptionIdentifier => {
                    let (id_len, sub_id) = length(bytes.iter())?;
                    // TODO: Validate 1 +. Tests are working either way
                    cursor += 1 + id_len;
                    bytes.advance(id_len);
                    id = Some(sub_id)
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

        Ok(Some(SubscribeProperties {
            subscription_identifier: id,
            user_properties,
        }))
    }

    pub fn write(
        properties: &SubscribeProperties,
        buffer: &mut BytesMut,
    ) -> Result<(), MQTTProtocolError> {
        let len = len(properties);
        write_remaining_length(buffer, len)?;

        if let Some(id) = &properties.subscription_identifier {
            buffer.put_u8(PropertyType::SubscriptionIdentifier as u8);
            write_remaining_length(buffer, *id)?;
        }

        for (key, value) in properties.user_properties.iter() {
            buffer.put_u8(PropertyType::UserProperty as u8);
            write_mqtt_string(buffer, key);
            write_mqtt_string(buffer, value);
        }

        Ok(())
    }
}
