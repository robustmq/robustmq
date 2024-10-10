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

pub fn write(unsubscribe: &Unsubscribe, buffer: &mut BytesMut) -> Result<usize, Error> {
    let remaining_len = 2 + unsubscribe // 2 bytes for packet identifier
        .filters
        .iter()
        .fold(0, |s, topic| s + topic.len() + 2); // the closure means s as accumlator with initial value zero
                                                  // and then accumulate each topic filter path length to s
                                                  // The final result will be the payload's length including
                                                  // length MSB+LSB for 2 bytes

    buffer.put_u8(0xA2);
    let remaining_len_bytes = write_remaining_length(buffer, remaining_len)?;
    buffer.put_u16(unsubscribe.pkid);

    for topic in unsubscribe.filters.iter() {
        write_mqtt_string(buffer, topic.as_str());
    }

    Ok(1 + remaining_len_bytes + remaining_len)
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<Unsubscribe, Error> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    let pkid = read_u16(&mut bytes)?;
    let mut payload_bytes = fixed_header.remaining_len - 2;
    let mut filters = Vec::with_capacity(1);

    while payload_bytes > 0 {
        let topic_filter = read_mqtt_string(&mut bytes)?;
        payload_bytes -= topic_filter.len() + 2;
        filters.push(topic_filter);
    }

    let unsubscribe = Unsubscribe { pkid, filters };

    Ok(unsubscribe)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_unsubscribe() {
        use super::*;
        // build filter and unsubscribe
        let mut buffer = BytesMut::new();

        let mut vec = Vec::new();
        let topic_1 = "test/topic1".to_string();
        let topic_2 = "test/topic2".to_string();
        vec.push(topic_1);
        vec.push(topic_2);

        let unsubscribe: Unsubscribe = Unsubscribe {
            pkid: 5u16,
            filters: vec,
        };

        // test unsubscribe write function
        write(&unsubscribe, &mut buffer);

        // test unsubscribe read function and verify the result of write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b1010_0010);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 28);
        let unsubscribe_read: Unsubscribe =
            read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(unsubscribe_read.pkid, 5u16);
        assert_eq!(unsubscribe_read.filters.len(), 2); // 2 topic filters pushed in the vector
    }
}
