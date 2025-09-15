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

impl PubComp {}

fn len() -> usize {
    2 // pkid which is packet identifier
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<PubComp, MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);
    let pkid = read_u16(&mut bytes)?;

    if fixed_header.remaining_len == 2 {
        Ok(PubComp {
            pkid,
            reason: Some(PubCompReason::Success),
        })
    } else {
        Err(MQTTProtocolError::InvalidRemainingLength(
            fixed_header.remaining_len,
        ))
    }
}
pub fn write(pubcomp: &PubComp, buffer: &mut BytesMut) -> Result<usize, MQTTProtocolError> {
    let len = len();
    buffer.put_u8(0x70);
    let count = write_remaining_length(buffer, len)?;
    buffer.put_u16(pubcomp.pkid);
    Ok(1 + count + len)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_pubcomp() {
        use super::*;
        let mut buffer = BytesMut::new();
        let pubcomp = PubComp {
            pkid: 5,
            reason: Some(PubCompReason::Success),
        };
        // test the write function of pubrec
        write(&pubcomp, &mut buffer).unwrap();

        // test the read function and verify the result of write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let pubcomp_read = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(fixed_header.byte1, 0b01110000);
        assert_eq!(fixed_header.remaining_len, 2);
        assert_eq!(pubcomp_read.pkid, 5u16);

        // test the display function of puback
        println!("pubcomp display: {pubcomp_read}");
    }
}
