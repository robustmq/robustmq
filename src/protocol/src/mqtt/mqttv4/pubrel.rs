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

impl PubRel {}

fn len() -> usize {
    2 //pkid which is publish packet identifier
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<PubRel, Error> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);
    let pkid = read_u16(&mut bytes)?;
    if fixed_header.remaining_len == 2 {
        Ok(PubRel {
            pkid,
            reason: Some(PubRelReason::Success),
        })
    } else {
        Err(Error::InvalidRemainingLength(fixed_header.remaining_len))
    }
}

pub fn write(pubrel: &PubRel, buffer: &mut BytesMut) -> Result<usize, Error> {
    let len = len();
    buffer.put_u8(0x62); // 1st byte of publish realease packet is 0b01100010
    let count = write_remaining_length(buffer, len)?;
    buffer.put_u16(pubrel.pkid);
    Ok(1 + count + len)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_pubrel() {
        use super::*;
        let mut buffer = BytesMut::new();
        let pubrel = PubRel
        // test the write function of pubrec
        write(&pubrel, &mut buffer);

        // test the read function and verify the result of write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        let pubrel_read = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(fixed_header.byte1, 0b01100010);
        assert_eq!(fixed_header.remaining_len, 2);
        assert_eq!(pubrel_read.pkid, 5u16);

        // test the display function of puback
        println!("pubrel display: {}", pubrel_read);
    }
}
