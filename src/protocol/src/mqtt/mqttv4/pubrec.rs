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

impl PubRec {}

fn len() -> usize {
    // pkid
    2
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<PubRec, Error> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);
    let pkid = read_u16(&mut bytes)?;
    if fixed_header.remaining_len == 2 {
        Ok(PubRec {
            pkid,
            reason: Some(PubRecReason::Success),
        })
    } else {
        Err(Error::InvalidRemainingLength(fixed_header.remaining_len))
    }
}

pub fn write(pubrec: &PubRec, buffer: &mut BytesMut) -> Result<usize, Error> {
    let len = len();
    buffer.put_u8(0x50);
    let count = write_remaining_length(buffer, len)?;
    buffer.put_u16(pubrec.pkid);
    Ok(1 + count + len)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_pubrec() {
        use super::*;
        let mut buffer = BytesMut::new();
        let pubrec = PubRec {
            pkid: 5,
            reason: Some(PubRecReason::Success),
        };
        // test the write function of pubrec
        write(&pubrec, &mut buffer).unwrap();

        // test the read function and verify the result of write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let pubrec_read = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(fixed_header.byte1, 0b01010000);
        assert_eq!(fixed_header.remaining_len, 2);
        assert_eq!(pubrec_read.pkid, 5u16);

        // test the display function of puback
        println!("pubrec display: {}", pubrec_read);
    }
}
