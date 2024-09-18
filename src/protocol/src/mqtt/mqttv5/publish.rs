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

/*
 * Copyright (c) 2023 robustmq team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use super::*;

pub fn write(
    publish: &Publish,
    properties: &Option<PublishProperties>,
    buffer: &mut BytesMut,
) -> Result<usize, Error> {
    let len = len(publish, properties);
    let dup = publish.dup as u8;
    let qos = publish.qos as u8;
    let retain = publish.retain as u8;
    buffer.put_u8(0b0011_0000 | retain | qos << 1 | dup << 3);

    let count = write_remaining_length(buffer, len)?;
    write_mqtt_bytes(buffer, &publish.topic);

    if publish.qos != QoS::AtMostOnce {
        let pkid = publish.pkid;
        if pkid == 0 {
            return Err(Error::PacketIdZero);
        }
        buffer.put_u16(pkid);
    }

    if let Some(p) = properties {
        properties::write(p, buffer)?;
    } else {
        write_remaining_length(buffer, 0);
    }

    buffer.extend_from_slice(&publish.payload);

    Ok(1 + count + len)
}

pub fn read(
    fixed_header: FixedHeader,
    mut bytes: Bytes,
) -> Result<(Publish, Option<PublishProperties>), Error> {
    let qos_num = (fixed_header.byte1 & 0b0110) >> 1;
    let qos = qos(qos_num).ok_or(Error::InvalidQoS(qos_num))?;
    let dup = (fixed_header.byte1 & 0b1000) != 0;
    let retain = (fixed_header.byte1 & 0b0001) != 0;

    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);
    let topic = read_mqtt_bytes(&mut bytes)?;
    // Packet identifier exists where QoS > 0
    let pkid = match qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce | QoS::ExactlyOnce => read_u16(&mut bytes)?,
    };

    if qos != QoS::AtMostOnce && pkid == 0 {
        return Err(Error::PacketIdZero);
    }

    let properties = properties::read(&mut bytes)?;
    let publish = Publish {
        dup,
        retain,
        qos,
        pkid,
        topic,
        payload: bytes,
    };

    Ok((publish, properties))
}

pub fn len(publish: &Publish, properties: &Option<PublishProperties>) -> usize {
    let mut len = 2 + publish.topic.len();

    // Publish identifier length if exists (QoS 1 or 2)
    if publish.qos != QoS::AtMostOnce && publish.pkid != 0 {
        len += 2;
    }

    if let Some(p) = properties {
        let properties_len = properties::len(p);
        let properties_len_len = len_len(properties_len);
        len += properties_len_len + properties_len;
    } else {
        // just 1 byte representing 0 len
        len += 1;
    }

    len += publish.payload.len();
    len
}

mod properties {
    use axum::extract::rejection::JsonDataError;

    use super::*;

    pub fn len(properties: &PublishProperties) -> usize {
        let mut len = 0;

        if properties.payload_format_indicator.is_some() {
            len += 1 + 1;
        }

        if properties.message_expiry_interval.is_some() {
            len += 1 + 4;
        }

        if properties.topic_alias.is_some() {
            len += 1 + 1;
        }

        if let Some(topic) = &properties.response_topic {
            len += 1 + 2 + topic.len();
        }

        if let Some(data) = &properties.correlation_data {
            len += 1 + 2 + data.len();
        }

        for (key, value) in properties.user_properties.iter() {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        for id in properties.subscription_identifiers.iter() {
            len += 1 + len_len(*id);
        }

        if let Some(content_type) = &properties.content_type {
            len += 1 + 2 + content_type.len();
        }

        len
    }

    pub fn write(properties: &PublishProperties, buffer: &mut BytesMut) -> Result<(), Error> {
        let len = len(properties);
        write_remaining_length(buffer, len)?;

        if let Some(payload_format_indicator) = properties.payload_format_indicator {
            buffer.put_u8(PropertyType::PayloadFormatIndicator as u8);
            buffer.put_u8(payload_format_indicator);
        }

        if let Some(message_expiry_interval) = properties.message_expiry_interval {
            buffer.put_u8(PropertyType::MessageExpiryInterval as u8);
            buffer.put_u32(message_expiry_interval);
        }

        if let Some(topic_alias) = properties.topic_alias {
            buffer.put_u8(PropertyType::TopicAlias as u8);
            buffer.put_u16(topic_alias);
        }

        if let Some(topic) = &properties.response_topic {
            buffer.put_u8(PropertyType::ResponseTopic as u8);
            write_mqtt_string(buffer, topic);
        }

        if let Some(data) = &properties.correlation_data {
            buffer.put_u8(PropertyType::CorrelationData as u8);
            write_mqtt_bytes(buffer, data);
        }

        for (key, value) in properties.user_properties.iter() {
            buffer.put_u8(PropertyType::UserProperty as u8);
            write_mqtt_string(buffer, key);
            write_mqtt_string(buffer, value);
        }

        for id in properties.subscription_identifiers.iter() {
            buffer.put_u8(PropertyType::SubscriptionIdentifier as u8);
            write_remaining_length(buffer, *id)?;
        }

        if let Some(content_type) = &properties.content_type {
            buffer.put_u8(PropertyType::ContentType as u8);
            write_mqtt_string(buffer, &content_type);
        }

        Ok(())
    }

    pub fn read(mut bytes: &mut Bytes) -> Result<Option<PublishProperties>, Error> {
        let mut payload_format_indicator = None;
        let mut message_expiry_interval = None;
        let mut topic_alias = None;
        let mut response_topic = None;
        let mut correlation_data = None;
        let mut user_properties = Vec::new();
        let mut subscription_identifiers = Vec::new();
        let mut content_type = None;

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
                PropertyType::PayloadFormatIndicator => {
                    payload_format_indicator = Some(read_u8(bytes)?);
                    cursor += 1;
                }

                PropertyType::MessageExpiryInterval => {
                    message_expiry_interval = Some(read_u32(bytes)?);
                    cursor += 4;
                }

                PropertyType::TopicAlias => {
                    topic_alias = Some(read_u16(bytes)?);
                    cursor += 2;
                }

                PropertyType::ResponseTopic => {
                    let topic = read_mqtt_string(bytes)?;
                    cursor += 2 + topic.len();
                    response_topic = Some(topic);
                }

                PropertyType::CorrelationData => {
                    let data = read_mqtt_bytes(bytes)?;
                    cursor += 2 + data.len();
                    correlation_data = Some(data);
                }

                PropertyType::UserProperty => {
                    let key = read_mqtt_string(bytes)?;
                    let value = read_mqtt_string(bytes)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }

                PropertyType::SubscriptionIdentifier => {
                    let (id_len, id) = length(bytes.iter())?;
                    cursor += 1 + id_len;
                    bytes.advance(id_len);
                    subscription_identifiers.push(id);
                }

                PropertyType::ContentType => {
                    let typ = read_mqtt_string(bytes)?;
                    cursor += 2 + typ.len();
                    content_type = Some(typ);
                }
                _ => return Err(Error::InvalidPropertyType(prop)),
            }
        }

        Ok(Some(PublishProperties {
            payload_format_indicator,
            message_expiry_interval,
            topic_alias,
            response_topic,
            correlation_data,
            user_properties,
            subscription_identifiers,
            content_type,
        }))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_publish_v5() {
        use super::*;
        let mut buffer: BytesMut = BytesMut::new();
        let topic_name: Bytes = Bytes::from("test_topic");
        let payload_value: Bytes = Bytes::from("test_payload");
        let retain_flag: bool = false;
        let mut publish: Publish = Publish::new(topic_name, payload_value, retain_flag);
        publish.qos = QoS::AtLeastOnce;
        publish.dup = true;
        publish.pkid = 15u16;
        // let publish_new = Publish {
        //     dup: true,
        //     qos: QoS::AtLeastOnce,
        //     ..publish
        // };

        let pub_properties: PublishProperties = PublishProperties {
            payload_format_indicator: Some(1u8),
            message_expiry_interval: Some(30u32),
            topic_alias: Some(20u16),
            response_topic: Some(String::from("response_topic")),
            correlation_data: None,
            user_properties: Vec::new(),
            subscription_identifiers: Vec::new(),
            content_type: Some("String".to_string()),
        };

        // test write function of publish in MQTT v5
        write(&publish, &Some(pub_properties), &mut buffer);

        // test the fixed_header part
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b00111010);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 62);

        // test the read function of publish packet and check the result of write function in MQTT v5
        let (x, y) = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(x.dup, true);
        assert_eq!(x.qos, QoS::AtLeastOnce);
        assert_eq!(x.pkid, 15);
        assert_eq!(x.topic, "test_topic");
        assert_eq!(x.payload, "test_payload");
        assert_eq!(x.retain, false);

        let pub_properties_read = y.unwrap();
        assert_eq!(pub_properties_read.payload_format_indicator, Some(1u8));
        assert_eq!(pub_properties_read.message_expiry_interval, Some(30u32));
        assert_eq!(pub_properties_read.topic_alias, Some(20u16));
        assert_eq!(
            pub_properties_read.response_topic,
            Some("response_topic".to_string())
        );
        assert_eq!(pub_properties_read.content_type, Some("String".to_string()));

        // test the display of publish packet
        println!("publish display in v5: {}", publish);
        println!("publish properties in v5: {}", pub_properties_read);
    }
}
