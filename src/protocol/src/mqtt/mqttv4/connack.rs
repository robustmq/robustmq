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

fn len() -> usize {
    // variable header length of connack is 2 bytes(session present + return code)
    1 + 1
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<ConnAck, Error> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    let flags = read_u8(&mut bytes)?;
    let return_code = read_u8(&mut bytes)?;
    let session_present = (flags & 0x01) == 1;
    let code = connect_return(return_code)?;
    let connack = ConnAck {
        session_present,
        code,
    };
    Ok(connack)
}

pub fn write(connack: &ConnAck, buffer: &mut BytesMut) -> Result<usize, Error> {
    let len = len();
    buffer.put_u8(0x20); // write the first byte 0010 0000 to demonstrate this packet type
                         //as a connack(connection acknowledgement)

    let count = write_remaining_length(buffer, len)?;
    buffer.put_u8(connack.session_present as u8);
    buffer.put_u8(connect_code(connack.code));

    Ok(1 + count + len)
}
fn connect_return(num: u8) -> Result<ConnectReturnCode, Error> {
    match num {
        0 => Ok(ConnectReturnCode::Success),
        1 => Ok(ConnectReturnCode::RefusedProtocolVersion),
        2 => Ok(ConnectReturnCode::BadClientId),
        3 => Ok(ConnectReturnCode::ServiceUnavailable),
        4 => Ok(ConnectReturnCode::BadUserNamePassword),
        5 => Ok(ConnectReturnCode::NotAuthorized),
        num => Err(Error::InvalidConnectReturnCode(num)),
    }
}

fn connect_code(return_code: ConnectReturnCode) -> u8 {
    match return_code {
        ConnectReturnCode::Success => 0,
        ConnectReturnCode::RefusedProtocolVersion => 1,
        ConnectReturnCode::BadClientId => 2,
        ConnectReturnCode::ServiceUnavailable => 3,
        ConnectReturnCode::BadUserNamePassword => 4,
        ConnectReturnCode::NotAuthorized => 5,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_connack() {
        use super::*;

        let connack: ConnAck = ConnAck {
            session_present: false,
            code: ConnectReturnCode::Success,
        };
        let mut buffer = BytesMut::new();
        // test the write function
        write(&connack, &mut buffer).unwrap();

        let fixedheader: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        // read the 1st byte and check its packet type which should be connack(0x20)
        assert_eq!(fixedheader.byte1, 0b0010_0000);
        // fixed header length should be 2
        assert_eq!(fixedheader.fixed_header_len, 2);
        // read the 2nd byte and check its value which should be 2
        assert_eq!(fixedheader.remaining_len, 2);

        // test the read function
        let connack_return = read(fixedheader, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert!(!connack_return.session_present);
        assert_eq!(connack_return.code, ConnectReturnCode::Success);
        // test the display function
        println!("connack display: {connack}");
    }
}
