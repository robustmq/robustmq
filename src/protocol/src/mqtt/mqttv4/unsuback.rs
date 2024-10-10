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

pub fn write(unsuback: &UnsubAck, buffer: &mut BytesMut) -> Result<usize, Error> {
    buffer.put_slice(&[0xB0, 0x02]);
    buffer.put_u16(unsuback.pkid);
    Ok(4)
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<UnsubAck, Error> {
    if fixed_header.remaining_len != 2 {
        return Err(Error::PayloadSizeIncorrect);
    }

    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);
    let pkid = read_u16(&mut bytes)?;
    let unsuback = UnsubAck {
        pkid,
        reasons: vec![],
    };

    Ok(unsuback)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_unsuback() {
        use super::*;
        let mut buffer = BytesMut::new();
        let unsuback = UnsubAck {
            pkid: 5u16,
            reasons: vec![], // reason is only valid in MQTT V5
        };

        // test unsuback write function
        write(&unsuback, &mut buffer);

        // test read and verify the result of the write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b1011_0000);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 2);
        let unsuback = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(unsuback.pkid, 5u16);
    }
}
