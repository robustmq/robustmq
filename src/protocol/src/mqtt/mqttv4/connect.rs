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

fn len(connect: &Connect, login: &Option<Login>, will: &Option<LastWill>) -> usize {
    /*
     * len is the variable header length which consists of four fields in the following order:
     * Protocol Name, Protocol Version, Connect Flags, and Keep Alive.
     * The first 2 bytes means MSB (the 1st one 0x0) and LSB (the 2nd one 0x4 - 4bytes means the length of "MQTT").
     * The 3rd byte to the 6th one are fixed with MQTT, which illustrates the protocol is MQTT
     */
    let mut len = 2 + "MQTT".len()      // protocol name
                              + 1       // protocol version
                              + 1       // connect flags
                              + 2; // keep alive

    //check later what the following 2 bytes mean for client id, for length?
    len += 2 + connect.client_id.len(); // client id length is mandatory and resides at the very beginning of payload

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
) -> Result<(u8, Connect, Option<Login>, Option<LastWill>), Error> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    //variable header
    let protocol_name = read_mqtt_bytes(&mut bytes)?;
    let protocol_name = std::str::from_utf8(&protocol_name)?.to_owned(); // convert to string by to_owned()
    let protocol_level = read_u8(&mut bytes)?;
    if protocol_name != "MQTT" {
        return Err(Error::InvalidProtocol);
    }

    if protocol_level != 4 && protocol_level != 3 {
        return Err(Error::InvalidProtocolLevel(protocol_level));
    }

    let connect_flags = read_u8(&mut bytes)?;
    let clean_session = (connect_flags & 0b10) != 0;
    let keep_alive = read_u16(&mut bytes)?;

    let client_id = read_mqtt_bytes(&mut bytes)?;
    let client_id = std::str::from_utf8(&client_id)?.to_owned();
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
) -> Result<usize, Error> {
    let len = self::len(connect, login, will);
    buffer.put_u8(0b0001_0000); // fixheader byte1 0x10
    let count = write_remaining_length(buffer, len)?;
    write_mqtt_string(buffer, "MQTT");

    buffer.put_u8(0x04);
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
        len += 2 + will.topic.len() + 2 + will.message.len();
        len
    }

    pub fn read(connect_flags: u8, bytes: &mut Bytes) -> Result<Option<LastWill>, Error> {
        let last_will = match connect_flags & 0b100 {
            // & 0b100 to check Will Flag(bit 2) is 0 or 1
            0 if (connect_flags & 0b0011_1000) != 0 => {
                // when Will Flag is 0, then "if" part checks
                // Will QoS(bit 3 & 4) and Will Retain(bit 5)
                return Err(Error::IncorrectPacketFormat); // if Will QoS and Retain are 1 but Will Flag 0, return incorrect
            }
            0 => None,
            _ => {
                let will_topic = read_mqtt_bytes(bytes)?;
                let will_message = read_mqtt_bytes(bytes)?;
                // figure out Will QoS number (0, 1 or 2) by &0b11000 and moving right 3 bits
                let qos_num = (connect_flags & 0b11000) >> 3;
                let will_qos = qos(qos_num).ok_or(Error::InvalidQoS(qos_num))?;
                //construct the LastWill message with topic name, message, QoS(0, 1 or 2) and Retain(0 or 1)
                Some(LastWill {
                    topic: will_topic,
                    message: will_message,
                    qos: will_qos,
                    retain: (connect_flags & 0b0010_00000) != 0,
                })
            }
        };

        Ok(last_will)
    }

    pub fn write(will: &LastWill, buffer: &mut BytesMut) -> Result<u8, Error> {
        let mut connect_flags = 0;

        connect_flags |= 0x04 | (will.qos as u8) << 3;
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

    pub fn read(connect_flags: u8, bytes: &mut Bytes) -> Result<Option<Login>, Error> {
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
            len += 2 + login.username.len();
        }

        if !login.password.is_empty() {
            len += 2 + login.password.len();
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
        println!("connect packet: {:?}", buff_write);
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
        println!("fixheader: {}", fixheader);
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

        let client_id = String::from("test_client_id");

        let login: Login = Login {
            username: String::from("test_user"),
            password: String::from("test_password"),
        };

        let will_topic = Bytes::from("will_topic");
        let will_message = Bytes::from("will_message");
        let lastwill: LastWill = LastWill {
            topic: will_topic,
            message: will_message,
            qos: QoS::AtLeastOnce,
            retain: true,
        };

        let connect: Connect = Connect {
            keep_alive: 30u16, // 30 seconds
            client_id,
            clean_session: true,
        };
        println!("connect display starts...........................");
        print!("{}", login);
        print!("{}", connect);
        println!("{}", lastwill);
        println!("connect display ends.............................");
    }
}
