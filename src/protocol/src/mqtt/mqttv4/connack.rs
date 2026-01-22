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

const MQTT_CONTROL_PACKET_TYPE_CONNACK: u8 = 0b0010_0000;
const MQTT_CONNECT_ACKNOWLEDGE_FLAGS_LENGTH: usize = 1;
const MQTT_CONNECT_RETURN_CODE_LENGTH: usize = 1;

fn remaining_length() -> usize {
    // variable header length of connack is 2 bytes(connect acknowledge flags + connect return code)
    MQTT_CONNECT_ACKNOWLEDGE_FLAGS_LENGTH + MQTT_CONNECT_RETURN_CODE_LENGTH
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<ConnAck, MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    let flags = read_u8(&mut bytes)?;
    let return_code = read_u8(&mut bytes)?;

    let session_present = (flags & 0x01) == 1;
    let code = connect_return(return_code)?;

    Ok(ConnAck {
        session_present,
        code,
    })
}

pub fn write(connack: &ConnAck, buffer: &mut BytesMut) -> Result<usize, MQTTProtocolError> {
    // write the first byte 0010 0000 to demonstrate this packet type
    //as a connack(connection acknowledgement)
    buffer.put_u8(MQTT_CONTROL_PACKET_TYPE_CONNACK);
    let len = remaining_length();
    let count = write_remaining_length(buffer, len)?;

    // variable header
    buffer.put_u8(connack.session_present as u8);
    buffer.put_u8(connect_code(connack.code));

    Ok(1 + count + len)
}
fn connect_return(num: u8) -> Result<ConnectReturnCode, MQTTProtocolError> {
    match num {
        0 => Ok(ConnectReturnCode::Success),
        1 => Ok(ConnectReturnCode::UnacceptableProtocolVersion),
        2 => Ok(ConnectReturnCode::IdentifierRejected),
        3 => Ok(ConnectReturnCode::ServiceUnavailable),
        4 => Ok(ConnectReturnCode::BadUserNamePassword),
        5 => Ok(ConnectReturnCode::NotAuthorized),
        num => Err(MQTTProtocolError::InvalidConnectReturnCode(num)),
    }
}

fn connect_code(return_code: ConnectReturnCode) -> u8 {
    match return_code {
        ConnectReturnCode::Success => 0,
        ConnectReturnCode::UnacceptableProtocolVersion => 1,
        ConnectReturnCode::IdentifierRejected => 2,
        ConnectReturnCode::ServiceUnavailable => 3,
        ConnectReturnCode::BadUserNamePassword => 4,
        ConnectReturnCode::NotAuthorized => 5,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::mqtt::common::ConnAck;
    use crate::mqtt::mqttv4::connack::MQTTProtocolError;
    use crate::mqtt::mqttv4::connack::{read, write};
    use crate::mqtt::mqttv4::parse_fixed_header;
    use crate::mqtt::mqttv4::FixedHeader;
    use bytes::{Buf, BytesMut};
    // use byte test do not use connack struct
    #[tokio::test]
    async fn test_connack_invalid_return_code() {
        let mut bytes = BytesMut::from(&[0x20, 0x02, 0x00, 0x06][..]);
        let fixedheader: FixedHeader = parse_fixed_header(bytes.iter()).unwrap();
        let connack_return = read(fixedheader, bytes.copy_to_bytes(bytes.len()));
        assert!(connack_return.is_err());
        let connack_error = connack_return.unwrap_err();
        assert_eq!(
            connack_error.to_string(),
            MQTTProtocolError::InvalidConnectReturnCode(6).to_string()
        )
    }

    // use byte to test read and write function
    #[tokio::test]
    async fn test_connack_read_write() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::Success,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(connack_return.session_present, connack.session_present);
        assert_eq!(connack_return.code, connack.code);
    }

    // test fixed header, variable header and payload
    #[tokio::test]
    async fn test_connack_fixed_header_length() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::Success,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();

        // read the 1st byte and check its packet type which should be connack(0x20)
        assert_eq!(fixedheader.byte1, 0b0010_0000);
        // fixed header length should be 2
        assert_eq!(fixedheader.fixed_header_len, 2);
        // read the 2nd byte and check its value which should be 2
        assert_eq!(fixedheader.remaining_len, 2);
    }

    #[tokio::test]
    async fn test_connack_variable_header() {
        let connack = ConnAck {
            session_present: true,
            code: super::ConnectReturnCode::Success,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert!(connack_return.session_present);
        assert_eq!(connack_return.code, super::ConnectReturnCode::Success);
    }

    // todo  this test can be improved to cover all return codes, but we need use actual mqtt client to test it
    #[tokio::test]
    async fn test_connack_connect_session_present_is_false() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::Success,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert!(!connack_return.session_present);
    }

    #[tokio::test]
    async fn test_connack_connect_session_present_is_true() {
        let connack = ConnAck {
            session_present: true,
            code: super::ConnectReturnCode::Success,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert!(connack_return.session_present);
    }

    #[tokio::test]
    async fn test_connack_connect_return_code_is_default() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::Success,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(connack_return.code, super::ConnectReturnCode::Success);
    }

    #[tokio::test]
    async fn test_connack_connect_return_code_is_refused_protocol_version() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::UnacceptableProtocolVersion,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(
            connack_return.code,
            super::ConnectReturnCode::UnacceptableProtocolVersion
        );
    }

    #[tokio::test]
    async fn test_connack_connect_return_code_is_bad_client_id() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::IdentifierRejected,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(connack_return.code, super::ConnectReturnCode::IdentifierRejected);
    }

    #[tokio::test]
    async fn test_connack_connect_return_code_is_service_unavailable() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::ServiceUnavailable,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(
            connack_return.code,
            super::ConnectReturnCode::ServiceUnavailable
        );
    }

    #[tokio::test]
    async fn test_connack_connect_return_code_is_bad_username_password() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::BadUserNamePassword,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(
            connack_return.code,
            super::ConnectReturnCode::BadUserNamePassword
        );
    }

    #[tokio::test]
    async fn test_connack_connect_return_code_is_bad_no_authorized() {
        let connack = ConnAck {
            session_present: false,
            code: super::ConnectReturnCode::NotAuthorized,
        };

        let mut buffer = BytesMut::new();
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(connack_return.code, super::ConnectReturnCode::NotAuthorized);
    }
}
