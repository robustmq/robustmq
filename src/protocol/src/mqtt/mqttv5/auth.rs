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

use common_base::error::mqtt_protocol_error::MQTTProtocolError;

use super::*;

pub fn len(auth: &Auth, properties: &Option<AuthProperties>) -> usize {
    // The Reason Code and Property Length can be omitted if the Reason Code is 0x00(Success)
    // and there are no properties. In this case the AUTH packet has a remaining length of 2.
    // <https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901217>
    if auth.reason.unwrap() == AuthReason::Success && properties.is_none() {
        return 2; // Packet type + 0x00
    }

    let mut len = 0;
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

pub fn write(
    auth: &Auth,
    properties: &Option<AuthProperties>,
    buffer: &mut BytesMut,
) -> Result<usize, MQTTProtocolError> {
    let len = len(auth, properties);
    buffer.put_u8(0b1111_0000);

    if len == 2 {
        buffer.put_u8(0x00); // reason code Success (0x00)
        return Ok(len);
    }
    let count = write_remaining_length(buffer, len)?;

    buffer.put_u8(code(auth.reason.unwrap()));

    if let Some(p) = &properties {
        properties::write(p, buffer)?;
    } else {
        write_remaining_length(buffer, 0)?;
    }

    Ok(1 + count + len)
}

pub fn read(
    fixed_header: FixedHeader,
    mut bytes: Bytes,
) -> Result<(Auth, Option<AuthProperties>), MQTTProtocolError> {
    let packet_type = fixed_header.byte1 >> 4;
    let flags = fixed_header.byte1 & 0b0000_1111;

    bytes.advance(fixed_header.fixed_header_len);

    if packet_type != PacketType::Auth as u8 {
        return Err(MQTTProtocolError::InvalidPacketType(packet_type));
    };

    if flags != 0x00 {
        return Err(MQTTProtocolError::MalformedPacket);
    };

    if fixed_header.remaining_len == 0 {
        return Ok((
            Auth {
                reason: Some(AuthReason::Success),
            },
            None,
        ));
    }

    let reason_code = read_u8(&mut bytes)?;

    let auth = Auth {
        reason: Some(reason(reason_code)?),
    };
    let properties = properties::read(&mut bytes)?;

    Ok((auth, properties))
}

mod properties {

    use super::*;

    pub fn len(properties: &AuthProperties) -> usize {
        let mut len = 0;

        if let Some(authentication_method) = &properties.authentication_method {
            len += 1 + 2 + authentication_method.len();
        }

        if let Some(authentication_data) = &properties.authentication_data {
            len += 1 + 2 + authentication_data.len();
        }

        if let Some(reason) = &properties.reason_string {
            len += 1 + 2 + reason.len();
        }

        for (key, value) in properties.user_properties.iter() {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        len
    }

    pub fn write(
        properties: &AuthProperties,
        buffer: &mut BytesMut,
    ) -> Result<(), MQTTProtocolError> {
        let len = len(properties);
        write_remaining_length(buffer, len)?;

        // write the authentication method
        if let Some(authentication_method) = &properties.authentication_method {
            // write the identifier of authentication method 21(0x15) - one byte
            buffer.put_u8(PropertyType::AuthenticationMethod as u8);
            // write the content of authentication method
            write_mqtt_string(buffer, authentication_method);
        }

        if let Some(authentication_data) = &properties.authentication_data {
            // write the identifier of authentication data 22(0x16) - one byte
            buffer.put_u8(PropertyType::AuthenticationData as u8);
            // write the content of authentication data
            write_mqtt_bytes(buffer, authentication_data);
        }

        if let Some(reason) = &properties.reason_string {
            // write the identifier of reason string 31(0x16) - one byte
            buffer.put_u8(PropertyType::ReasonString as u8);
            // write the content of reason string
            write_mqtt_string(buffer, reason);
        }

        for (key, value) in properties.user_properties.iter() {
            // write the identifier of user properties 38(0x26) - one byte
            buffer.put_u8(PropertyType::UserProperty as u8);
            // write the key of user properties
            write_mqtt_string(buffer, key);
            // write the value of user properties
            write_mqtt_string(buffer, value);
        }

        Ok(())
    }

    pub fn read(bytes: &mut Bytes) -> Result<Option<AuthProperties>, MQTTProtocolError> {
        let mut authentication_method = None;
        let mut authentication_data = None;
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
                PropertyType::AuthenticationMethod => {
                    let method = read_mqtt_string(bytes)?;
                    cursor += 2 + method.len();
                    authentication_method = Some(method);
                }

                PropertyType::AuthenticationData => {
                    let data = read_mqtt_bytes(bytes)?;
                    cursor += 2 + data.len();
                    authentication_data = Some(data);
                }

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

        Ok(Some(AuthProperties {
            authentication_method,
            authentication_data,
            reason_string,
            user_properties,
        }))
    }
}

fn code(reason: AuthReason) -> u8 {
    match reason {
        AuthReason::Success => 0x00,
        AuthReason::ContinueAuthentication => 0x18,
        AuthReason::ReAuthenticate => 0x19,
    }
}

fn reason(code: u8) -> Result<AuthReason, MQTTProtocolError> {
    let v = match code {
        0x00 => AuthReason::Success,
        0x18 => AuthReason::ContinueAuthentication,
        0x19 => AuthReason::ReAuthenticate,
        other => return Err(MQTTProtocolError::InvalidConnectReturnCode(other)),
    };
    Ok(v)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_auth_v5() {
        use super::*;

        let mut buffer = BytesMut::new();
        let auth = Auth {
            reason: Some(AuthReason::ContinueAuthentication),
        };

        let authentication_method: String = "SCRAM-SHA-256".to_string();
        let authentication_data: Bytes = Bytes::from("client-first-data");
        let user_properties: Vec<(String, String)> = vec![
            ("username".into(), "Justin".into()),
            ("tag".to_string(), "middleware".to_string()),
        ];

        let properties = AuthProperties {
            authentication_method: Some(authentication_method),
            authentication_data: Some(authentication_data),
            reason_string: Some(String::from("Not Success")),
            user_properties,
        };

        // test the write function of Auth in v5
        write(&auth, &Some(properties.clone()), &mut buffer).unwrap();

        // test the fixed_header part
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b1111_0000);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 88);

        // test the read function of pubrec packet and check the result of write function in MQTT v5
        let (auth_read, y) = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(
            auth_read.reason.unwrap(),
            AuthReason::ContinueAuthentication
        );

        let auth_properties = y.unwrap();
        assert_eq!(
            auth_properties.reason_string,
            Some("Not Success".to_string())
        );
        assert_eq!(
            auth_properties.user_properties.first(),
            Some(&("username".to_string(), "Justin".to_string()))
        );

        // test display of puback and puback_properties in v5
        assert_eq!(auth.to_string(), auth_read.to_string());
        assert_eq!(properties.to_string(), auth_properties.to_string());
    }
}
