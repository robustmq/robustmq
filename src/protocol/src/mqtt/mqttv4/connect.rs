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

const MQTT_CONTROL_PACKET_TYPE_CONNECT: u8 = 0b0001_0000;
const MQTT_PROTOCOL_VERSION_3_1_1: u8 = 0b0000_0100;
const MQTT_LENGTH_MSB: usize = 1;
const MQTT_LENGTH_LSB: usize = 1;
const MQTT_PROTOCOL_NAME_DESCRIPTION_LENGTH: usize = MQTT_LENGTH_MSB + MQTT_LENGTH_LSB;
const MQTT_PROTOCOL_NAME_LENGTH: usize = "MQTT".len();
const MQTT_CONNECT_PROTOCOL_LEVEL_LENGTH: usize = 1;
const MQTT_CONNECT_ACKNOWLEDGE_FLAGS_LENGTH: usize = 1;
const MQTT_KEEP_ALIVE_DESCRIPTION_LENGTH: usize = 2;

fn remaining_length(connect: &Connect, login: &Option<Login>, will: &Option<LastWill>) -> usize {
    variable_header_length() + payload_length(connect, login, will)
}

fn variable_header_length() -> usize {
    /*
     * len is the variable header length which consists of four fields in the following order:
     * Protocol Name, Protocol Version, Connect Flags, and Keep Alive.
     * The first 2 bytes means MSB (the 1st one 0x0) and LSB (the 2nd one 0x4 - 4bytes means the length of "MQTT").
     * The 3rd byte to the 6th one are fixed with MQTT, which illustrates the protocol is MQTT
     */
    MQTT_PROTOCOL_NAME_DESCRIPTION_LENGTH
        + MQTT_PROTOCOL_NAME_LENGTH
        + MQTT_CONNECT_PROTOCOL_LEVEL_LENGTH
        + MQTT_CONNECT_ACKNOWLEDGE_FLAGS_LENGTH
        + MQTT_KEEP_ALIVE_DESCRIPTION_LENGTH
}

fn payload_length(connect: &Connect, login: &Option<Login>, will: &Option<LastWill>) -> usize {
    let mut len = 0;

    //  check later what the following 2 bytes mean for client id, for length?
    // client id length is mandatory and resides at the very beginning of payload
    len += MQTT_LENGTH_MSB;
    len += MQTT_LENGTH_LSB;
    len += connect.client_id.len();

    // last will length
    if let Some(w) = will {
        len += will::len(w);
    }

    // username and password length
    if let Some(l) = login {
        len += login::len(l);
    }

    len
}

pub fn read(
    fixed_header: FixedHeader,
    mut bytes: Bytes,
) -> Result<(u8, Connect, Option<Login>, Option<LastWill>), MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    //variable header
    let protocol_name = read_mqtt_bytes(&mut bytes)?;
    let protocol_name = std::str::from_utf8(&protocol_name)?.to_owned(); // convert to string by to_owned()
    if protocol_name != "MQTT" {
        return Err(MQTTProtocolError::InvalidProtocolName);
    }

    let protocol_level = read_u8(&mut bytes)?;
    if protocol_level != 4 && protocol_level != 3 {
        return Err(MQTTProtocolError::InvalidProtocolLevel(protocol_level));
    }

    let connect_flags = read_u8(&mut bytes)?;
    // todo check reserved bit (bit 0) is 0
    if (connect_flags & 0b1) != 0 {
        return Err(MQTTProtocolError::ReservedSetError);
    }
    let clean_session = (connect_flags & 0b10) != 0;
    let keep_alive = read_u16(&mut bytes)?;

    let client_id = read_mqtt_bytes(&mut bytes)?;
    // TODO: when clean session is 1, client id can be zero length
    // for this code, we will read and return "", for this case
    // maybe we need give a random client id later

    let client_id = std::str::from_utf8(&client_id)?.to_owned();
    if client_id.is_empty() && (connect_flags & 0b10) == 0 {
        return Err(MQTTProtocolError::IncorrectPacketFormat);
    }
    let last_will = will::read(connect_flags, &mut bytes)?;
    let login = login::read(connect_flags, &mut bytes)?;

    let connect = Connect {
        keep_alive,
        client_id,
        clean_session,
    };

    Ok((protocol_level, connect, login, last_will))
}

pub fn write(
    connect: &Connect,
    login: &Option<Login>,
    will: &Option<LastWill>,
    buffer: &mut BytesMut,
) -> Result<usize, MQTTProtocolError> {
    // fixed header
    buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT); // fixheader byte1 0x10
    let len = remaining_length(connect, login, will);

    let count = write_remaining_length(buffer, len)?;

    // variable header
    write_mqtt_string(buffer, "MQTT");
    buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
    let flags_index = 1 + count + 2 + 4 + 1;
    let mut connect_flags = 0;
    if connect.clean_session {
        connect_flags |= 0x02;
    }
    buffer.put_u8(connect_flags);

    buffer.put_u16(connect.keep_alive);

    write_mqtt_string(buffer, &connect.client_id);
    if let Some(w) = &will {
        connect_flags |= will::write(w, buffer)?;
    }

    if let Some(l) = login {
        connect_flags |= login::write(l, buffer);
    }

    // update connect flags
    buffer[flags_index] = connect_flags;
    Ok(len)
}

pub mod will {
    use super::*;

    pub fn len(will: &LastWill) -> usize {
        let mut len = 0;
        //check later what the following 2 bytes mean for will topic and message, for length?

        len += MQTT_LENGTH_MSB + MQTT_LENGTH_LSB; // will topic length
        len += will.topic.len();

        len += MQTT_LENGTH_MSB + MQTT_LENGTH_LSB; // will message length
        len += will.message.len();

        len
    }

    pub fn read(
        connect_flags: u8,
        bytes: &mut Bytes,
    ) -> Result<Option<LastWill>, MQTTProtocolError> {
        let last_will = match connect_flags & 0b100 {
            // & 0b100 to check Will Flag(bit 2) is 0 or 1
            0 if (connect_flags & 0b0011_1000) != 0 => {
                // when Will Flag is 0, then "if" part checks
                // Will QoS(bit 3 & 4) and Will Retain(bit 5)
                return Err(MQTTProtocolError::IncorrectPacketFormat); // if Will QoS and Retain are 1 but Will Flag 0, return incorrect
            }
            0 => None,
            _ => {
                let will_topic = read_mqtt_bytes(bytes)?;
                let will_message = read_mqtt_bytes(bytes)?;
                // figure out Will QoS number (0, 1 or 2) by &0b11000 and moving right 3 bits
                let qos_num = (connect_flags & 0b11000) >> 3;
                let will_qos = qos(qos_num).ok_or(MQTTProtocolError::InvalidQoS(qos_num))?;
                //construct the LastWill message with topic name, message, QoS(0, 1 or 2) and Retain(0 or 1)
                Some(LastWill {
                    topic: will_topic,
                    message: will_message,
                    qos: will_qos,
                    retain: (connect_flags & 0b0010_0000) != 0,
                })
            }
        };

        Ok(last_will)
    }

    pub fn write(will: &LastWill, buffer: &mut BytesMut) -> Result<u8, MQTTProtocolError> {
        let mut connect_flags = 0;

        connect_flags |= 0x04 | ((will.qos as u8) << 3);
        if will.retain {
            connect_flags |= 0x20;
        }

        write_mqtt_bytes(buffer, &will.topic);
        write_mqtt_bytes(buffer, &will.message);
        Ok(connect_flags)
    }
}

pub mod login {
    use super::*;

    pub fn read(connect_flags: u8, bytes: &mut Bytes) -> Result<Option<Login>, MQTTProtocolError> {
        // if username is zero and password is not zero, return incorrect packet format
        if (connect_flags & 0b1000_0000) == 0 && (connect_flags & 0b0100_0000) != 0 {
            return Err(MQTTProtocolError::IncorrectPacketFormat);
        }
        let username = match connect_flags & 0b1000_0000 {
            0 => String::new(),
            _ => {
                let username = read_mqtt_bytes(bytes)?;
                std::str::from_utf8(&username)?.to_owned()
            }
        };

        let password = match connect_flags & 0b0100_0000 {
            0 => String::new(),
            _ => {
                let password = read_mqtt_bytes(bytes)?;
                std::str::from_utf8(&password)?.to_owned()
            }
        };

        if username.is_empty() && password.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Login { username, password }))
        }
    }

    pub fn len(login: &Login) -> usize {
        let mut len = 0;

        if !login.username.is_empty() {
            len += MQTT_LENGTH_MSB;
            len += MQTT_LENGTH_LSB;
            len += login.username.len();
        }

        if !login.password.is_empty() {
            len += MQTT_LENGTH_MSB;
            len += MQTT_LENGTH_LSB;
            len += login.password.len();
        }

        len
    }

    pub fn write(login: &Login, buffer: &mut BytesMut) -> u8 {
        let mut connect_flags = 0;
        if !login.username.is_empty() {
            connect_flags |= 0x80;
            write_mqtt_string(buffer, &login.username);
        }

        if !login.password.is_empty() {
            connect_flags |= 0x40;
            write_mqtt_string(buffer, &login.password);
        }

        connect_flags
    }

    pub fn validate(login: &Login, username: &str, password: &str) -> bool {
        (login.username == *username) && (login.password == *password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test fixed header, variable header and payload for connect packet
    #[tokio::test]
    async fn test_connect_fixed_header_length() {
        let connect = Connect {
            keep_alive: 0,
            client_id: "test_id".to_string(),
            clean_session: false,
        };

        let mut buffer = BytesMut::new();

        write(&connect, &None, &None, &mut buffer).unwrap();

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();

        // connect packet type
        assert_eq!(fixed_header.byte1, 0b0001_0000);
        // fixed header length
        assert_eq!(fixed_header.fixed_header_len, 2);
        // remaining length
        assert_eq!(
            fixed_header.remaining_len,
            variable_header_length() + 2 + connect.client_id.len()
        );
    }

    #[tokio::test]
    async fn test_connect_variable_header() {
        let connect = Connect {
            keep_alive: 0,
            client_id: "test_id".to_string(),
            clean_session: false,
        };

        let mut buffer = BytesMut::new();
        write(&connect, &None, &None, &mut buffer).unwrap();
        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let (protocol_level, connect, login, lass_will) =
            read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();

        assert_eq!(protocol_level, MQTT_PROTOCOL_VERSION_3_1_1); // protocol level
        assert_eq!(connect.keep_alive, connect.keep_alive);
        assert_eq!(connect.client_id, connect.client_id);
        assert_eq!(connect.clean_session, connect.clean_session);
        assert!(login.is_none());
        assert!(lass_will.is_none());
    }

    #[tokio::test]
    async fn variable_header_should_fail_with_invalid_protocol_name() {
        let mut buffer = BytesMut::new();
        // Manually write an invalid protocol name
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8(10); // Remaining length
        write_mqtt_string(&mut buffer, "INVALID");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        buffer.put_u8(0); // Connect flags
        buffer.put_u16(0); // Keep alive
        write_mqtt_string(&mut buffer, "test_id");

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));

        assert!(matches!(
            result,
            Err(MQTTProtocolError::InvalidProtocolName)
        ));
    }

    #[tokio::test]
    async fn variable_header_should_fail_with_invalid_protocol_level() {
        let mut buffer = BytesMut::new();
        // Manually write an invalid protocol name
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8(10); // Remaining length
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(0b0000_0101); // Invalid Protocol Level
        buffer.put_u8(0); // Connect flags
        buffer.put_u16(0); // Keep alive
        write_mqtt_string(&mut buffer, "test_id");

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));
        assert!(matches!(
            result,
            Err(MQTTProtocolError::InvalidProtocolLevel(5))
        ));
    }

    #[tokio::test]
    async fn variable_header_should_fail_when_user_name_flag_is_zero_and_password_flag_is_not_zero()
    {
        let mut buffer = BytesMut::new();
        // Manually write an invalid protocol name
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8(10); // Remaining length
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        buffer.put_u8(0b0100_0000); // Connect flags with Username Flag 0 but Password Flag 1
        buffer.put_u16(0); // Keep alive
        write_mqtt_string(&mut buffer, "test_id");

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));

        assert!(matches!(
            result,
            Err(MQTTProtocolError::IncorrectPacketFormat)
        ));
    }

    #[tokio::test]
    async fn variable_header_should_fail_when_will_flag_is_zero_and_will_retain_is_not_zero() {
        let mut buffer = BytesMut::new();
        // Manually write an invalid protocol name
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8(10); // Remaining length
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        buffer.put_u8(0b0010_0000); // Connect flags with Will Flag 0 but Will Retain not 0
        buffer.put_u16(0); // Keep alive
        write_mqtt_string(&mut buffer, "test_id");

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));

        assert!(matches!(
            result,
            Err(MQTTProtocolError::IncorrectPacketFormat)
        ));
    }

    #[tokio::test]
    async fn variable_header_should_fail_when_will_flag_is_zero_and_will_qos_is_not_zero() {
        let mut buffer = BytesMut::new();
        // Manually write an invalid protocol name
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8(10); // Remaining length
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        // Connect flags with Will Flag 0 but Will QoS not 0
        buffer.put_u8(0b0000_1000);
        buffer.put_u16(0); // Keep alive
        write_mqtt_string(&mut buffer, "test_id");

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));

        assert!(matches!(
            result,
            Err(MQTTProtocolError::IncorrectPacketFormat)
        ));
    }

    #[tokio::test]
    async fn variable_header_should_fail_with_invalid_qos_value() {
        let mut buffer = BytesMut::new();
        // Manually write an invalid protocol name
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8(10); // Remaining length
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        // Connect flags with Will Flag 1 and invalid Will QoS 3
        buffer.put_u8(0b0001_1100);
        buffer.put_u16(0); // Keep alive
        write_mqtt_string(&mut buffer, "test_id");
        // Will Topic
        write_mqtt_string(&mut buffer, "will_topic");
        // Will Message
        write_mqtt_string(&mut buffer, "will_message");

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));

        assert!(matches!(result, Err(MQTTProtocolError::InvalidQoS(3))));
    }

    #[tokio::test]
    async fn variable_header_should_fail_with_reserved_flags_set_is_one() {
        let mut buffer = BytesMut::new();
        // Manually write an invalid protocol name
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8(10); // Remaining length
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        buffer.put_u8(0b0000_0001); // Connect flags
        buffer.put_u16(0); // Keep alive
        write_mqtt_string(&mut buffer, "test_id");

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));

        assert!(matches!(result, Err(MQTTProtocolError::ReservedSetError)));
    }

    #[tokio::test]
    async fn payload_need_in_order_of_client_id_will_topic_will_message_username_and_password() {
        let client_id = "test_id";
        let will_topic = "will_topic";
        let will_message = "will_message";
        let user_name = "test_username";
        let user_password = "test_password";
        let pay_load_length = client_id.len()
            + will_topic.len()
            + will_message.len()
            + user_name.len()
            + user_password.len();

        let mut buffer = BytesMut::new();
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8((10 + pay_load_length) as u8); // Remaining length will be updated later
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        // Connect flags with Will Flag 1, Will QoS 1, Will Retain 1, Username Flag 1 and Password Flag 1
        buffer.put_u8(0b1110_1100);
        buffer.put_u16(0); // Keep alive

        // Client ID
        write_mqtt_string(&mut buffer, client_id);
        // Will Topic
        write_mqtt_string(&mut buffer, will_topic);
        // Will Message
        write_mqtt_string(&mut buffer, will_message);
        // Username
        write_mqtt_string(&mut buffer, user_name);
        // Password
        write_mqtt_string(&mut buffer, user_password);

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let (_protocol_level, connect, login, last_will) =
            read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(connect.client_id, client_id);
        let lw = last_will.unwrap();
        assert_eq!(std::str::from_utf8(&lw.topic).unwrap(), will_topic);
        assert_eq!(std::str::from_utf8(&lw.message).unwrap(), will_message);
        assert_eq!(lw.qos, QoS::AtLeastOnce);
        assert!(lw.retain);
        let lg = login.unwrap();
        assert_eq!(lg.username, user_name);
        assert_eq!(lg.password, user_password);
    }

    #[tokio::test]
    async fn client_id_can_set_zero_when_clean_session_set_1_in_payload() {
        let client_id = "";
        let will_topic = "will_topic";
        let will_message = "will_message";
        let user_name = "test_username";
        let user_password = "test_password";
        let pay_load_length = client_id.len()
            + will_topic.len()
            + will_message.len()
            + user_name.len()
            + user_password.len();

        let mut buffer = BytesMut::new();
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8((10 + pay_load_length) as u8); // Remaining length will be updated later
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        // Connect flags with Will Flag 1, Will QoS 1, Will Retain 1, Username Flag 1 and Password Flag 1
        // Clean Session 1
        buffer.put_u8(0b1110_1110);
        buffer.put_u16(0); // Keep alive

        // Client ID
        write_mqtt_string(&mut buffer, client_id);
        // Will Topic
        write_mqtt_string(&mut buffer, will_topic);
        // Will Message
        write_mqtt_string(&mut buffer, will_message);
        // Username
        write_mqtt_string(&mut buffer, user_name);
        // Password
        write_mqtt_string(&mut buffer, user_password);

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let (_protocol_level, connect, login, last_will) =
            read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(connect.client_id, client_id);
        let lw = last_will.unwrap();
        assert_eq!(std::str::from_utf8(&lw.topic).unwrap(), will_topic);
        assert_eq!(std::str::from_utf8(&lw.message).unwrap(), will_message);
        assert_eq!(lw.qos, QoS::AtLeastOnce);
        assert!(lw.retain);
        let lg = login.unwrap();
        assert_eq!(lg.username, user_name);
        assert_eq!(lg.password, user_password);
    }

    #[tokio::test]
    async fn client_id_can_not_set_zero_when_clean_session_set_0_in_payload() {
        let client_id = "";
        let will_topic = "will_topic";
        let will_message = "will_message";
        let user_name = "test_username";
        let user_password = "test_password";
        let pay_load_length = client_id.len()
            + will_topic.len()
            + will_message.len()
            + user_name.len()
            + user_password.len();

        let mut buffer = BytesMut::new();
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8((10 + pay_load_length) as u8); // Remaining length will be updated later
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        // Connect flags with Will Flag 1, Will QoS 1, Will Retain 1, Username Flag 1 and Password Flag 1
        // Clean Session 0
        buffer.put_u8(0b1110_1100);
        buffer.put_u16(0); // Keep alive

        // Client ID
        write_mqtt_string(&mut buffer, client_id);
        // Will Topic
        write_mqtt_string(&mut buffer, will_topic);
        // Will Message
        write_mqtt_string(&mut buffer, will_message);
        // Username
        write_mqtt_string(&mut buffer, user_name);
        // Password
        write_mqtt_string(&mut buffer, user_password);

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let result = read(fixed_header, buffer.copy_to_bytes(buffer.len()));

        assert!(matches!(
            result,
            Err(MQTTProtocolError::IncorrectPacketFormat)
        ));
    }

    #[tokio::test]
    async fn payload_has_will_topic_and_will_message_when_will_flag_set_1() {
        let client_id = "test_id";
        let will_topic = "will_topic";
        let will_message = "will_message";

        let pay_load_length = client_id.len() + will_topic.len() + will_message.len();

        let mut buffer = BytesMut::new();
        buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNECT);
        buffer.put_u8((10 + pay_load_length) as u8); // Remaining length will be updated later
        write_mqtt_string(&mut buffer, "MQTT");
        buffer.put_u8(MQTT_PROTOCOL_VERSION_3_1_1);
        // Connect flags with Will Flag 1, Will QoS 1, Will Retain 1
        buffer.put_u8(0b0010_1100);
        buffer.put_u16(0); // Keep alive

        // Client ID
        write_mqtt_string(&mut buffer, client_id);
        // Will Topic
        write_mqtt_string(&mut buffer, will_topic);
        // Will Message
        write_mqtt_string(&mut buffer, will_message);

        let fixed_header = parse_fixed_header(buffer.iter()).unwrap();
        let (_protocol_level, connect, login, last_will) =
            read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(connect.client_id, client_id);
        let lw = last_will.unwrap();
        assert_eq!(std::str::from_utf8(&lw.topic).unwrap(), will_topic);
        assert_eq!(std::str::from_utf8(&lw.message).unwrap(), will_message);
        assert_eq!(lw.qos, QoS::AtLeastOnce);
        println!("lw.retain = {}", lw.retain);
        assert!(lw.retain);
        assert!(login.is_none());
    }

    #[test]
    fn test_connect() {
        use super::*;

        let client_id = String::from("test_client_id");
        let client_id_length = client_id.len();
        let mut buff_write = BytesMut::new();
        //let mut buff = BytesMut::with_capacity(1024);
        let login = None;
        let lastwill = None;

        let connect: Connect = Connect {
            keep_alive: 30u16, // 30 seconds
            client_id,
            clean_session: true,
        };
        // test function write
        write(&connect, &login, &lastwill, &mut buff_write).unwrap();

        //construct the test case - connect packet: b"\x10\x1a\0\x04MQTT\x04\x02\0\x1e\0\x0etest_client_id”
        //The total length is 28 bytes. break it down into 3 parts - fixed header, variable header and payload

        //1.Fixed header —————————————————————— 2 bytes
        // b"\x10\x1a\ —————————————————————— 2 bytes(Fixed header) 00010000(packet type as connect)   26 bytes(remaining length)

        //2.Variable header ——————————————————— 10 bytes
        // 0\x04 ———————————————————————— 2 bytes (MSB 1 byte as 0, LSB 1 byte as 0x04)
        // MQTT ———————————————————————— 4 bytes (MQTT协议)
        // \x04\————————————————————————— 1 byte (MQTT version, x04 means v3.1.1, x05 means v5)
        // x02 ————————————————————————— 1 byte( connect flag , x02 means clean session as true )
        // \0\x1e ————————————————————————— 2 bytes ( keep_alive, 30 seconds in the test case)

        //3.Payload ———————————————————————— 16 bytes
        // \0\x0e ————————————————————————— 2 bytes( length of client id )
        // test_client_id ————————————————————— 14 bytes (client id)

        println!("buff size is {}", buff_write.len());
        println!("connect packet: {buff_write:?}");
        assert_eq!(buff_write.len(), 28); // check the total length of the packet
        assert_eq!(buff_write[0], 0b0001_0000); // check the packet type is 0x10 which is connect type
        assert_eq!(buff_write[1], 26); // check the remaining length (length of variable header + payload) right or not
        assert_eq!(buff_write[2], 0); // check the MSB which is 0
        assert_eq!(buff_write[3], 4); // check the LSB which is 4 (length of "MQTT")
        assert_eq!(&buff_write[4..8], b"MQTT"); // check the protocol which should be MQTT
                                                // (4..8 means starting with item index 4 and following 4 bytes(=8-4))
        assert_eq!(buff_write[8], 4); // check the protocol version which should be 4
        assert_eq!(buff_write[9], 0b00000010); // check the connect flags which only set clean session = true
        assert_eq!(u16::from_be_bytes([buff_write[10], buff_write[11]]), 30u16); // check the keep_alive value (set 60s in the test case)

        assert_eq!(
            u16::from_be_bytes([buff_write[12], buff_write[13]]),
            client_id_length as u16
        ); // check the client id length in payload

        assert_eq!(&buff_write[14..], b"test_client_id"); // check the client id

        // read the fixed header
        let fixheader: FixedHeader = parse_fixed_header(buff_write.iter()).unwrap();
        // test the display of fixheader
        println!("fixheader: {fixheader}");
        // read first byte and check its packet type which should be connect
        assert_eq!(fixheader.byte1, 0b0001_0000);
        assert_eq!(fixheader.fixed_header_len, 2);
        assert!(fixheader.remaining_len == 26);
        // test read function, x gets connect, y gets login and z gets will
        let (_, x, _, _) = read(fixheader, buff_write.copy_to_bytes(buff_write.len())).unwrap();
        // only check connect value in this case as login and will being none
        assert_eq!(x.client_id, "test_client_id");
        assert_eq!(x.keep_alive, 30);
        assert!(x.clean_session);
    }

    #[test]
    fn test_display() {
        use super::*;

        let login = Login {
            username: "test_username".to_string(),
            password: "test_password".to_string(),
        };

        let lastwill = LastWill {
            topic: Bytes::from("will_topic"),
            message: Bytes::from("will_message"),
            qos: QoS::AtLeastOnce,
            retain: true,
        };

        let connect = Connect {
            keep_alive: 30u16,
            client_id: "test_client_id".to_string(),
            clean_session: true,
        };

        assert_eq!(
            login.to_string(),
            "username:\"test_username\", password:\"test_password\" "
        );
        assert_eq!(
            lastwill.to_string(),
            "will_topic:b\"will_topic\", qos:AtLeastOnce, will_message:b\"will_message\", retain:true "
        );
        assert_eq!(
            connect.to_string(),
            "client_id:\"test_client_id\", clean_session:true, keep_alive:30s "
        );
    }
}
