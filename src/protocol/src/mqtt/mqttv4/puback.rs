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

/// puback packet is an acknowledgement to QoS 1 publish packet
use super::*;
use common_base::error::mqtt_protocol_error::MQTTProtocolError;

impl PubAck {}
fn len() -> usize {
    2 // pkid - publish identifier
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<PubAck, MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    if fixed_header.remaining_len != 2 {
        return Err(MQTTProtocolError::InvalidRemainingLength(
            fixed_header.remaining_len,
        ));
    }

    let pkid = read_u16(&mut bytes)?;
    let puback = PubAck {
        pkid,
        reason: Some(PubAckReason::Success),
    };

    Ok(puback)
}

pub fn write(puback: &PubAck, buffer: &mut BytesMut) -> Result<usize, MQTTProtocolError> {
    let len = len();
    buffer.put_u8(0x40);
    let count = write_remaining_length(buffer, len)?;
    buffer.put_u16(puback.pkid);
    Ok(1 + count + len)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_puback() {
        use super::*;

        let mut buffer: BytesMut = BytesMut::new();
        let puback: PubAck = PubAck {
            pkid: 1,
            reason: Some(PubAckReason::Success),
        };

        // test the write function
        write(&puback, &mut buffer).unwrap();

        // test the read function and verify the result of write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b01000000);
        let puback_read = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(puback_read.pkid, puback.pkid);
        assert_eq!(puback_read.reason, puback.reason);

        // test the display function of puback
        assert_eq!(puback.to_string(), puback_read.to_string());
    }
}
