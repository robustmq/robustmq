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

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<Publish, Error> {
    let qos_num = (fixed_header.byte1 & 0b0110) >> 1;
    let qos = qos(qos_num).ok_or(Error::InvalidQoS(qos_num))?;
    let dup = (fixed_header.byte1 & 0b1000) != 0;
    let retain = (fixed_header.byte1 & 0b0001) != 0;

    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);
    let topic = read_mqtt_bytes(&mut bytes)?;

    // Packet identifier exists where QoS > 0
    let pkid = match qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce | QoS::ExactlyOnce => read_u16(&mut bytes)?,
    };

    if qos != QoS::AtMostOnce && pkid == 0 {
        return Err(Error::PacketIdZero);
    }

    let publish = Publish {
        dup,
        retain,
        qos,
        pkid,
        topic,
        payload: bytes,
    };

    Ok(publish)
}

pub fn write(publish: &Publish, buffer: &mut BytesMut) -> Result<usize, Error> {
    let len = publish.len();
    let dup = publish.dup as u8;
    let qos = publish.qos as u8;
    let retain = publish.retain as u8;
    buffer.put_u8(0b0011_0000 | retain | qos << 1 | dup << 3);

    let count = write_remaining_length(buffer, len)?;
    write_mqtt_bytes(buffer, &publish.topic);

    if publish.qos != QoS::AtMostOnce {
        let pkid = publish.pkid;
        if pkid == 0 {
            return Err(Error::PacketIdZero);
        }
        buffer.put_u16(pkid);
    }
    buffer.extend_from_slice(&publish.payload);

    Ok(1 + count + len) // 1st byte for packet type and connect flag,
                        // count means how many bytes (1~4) to express the length of remaining length,
                        // len menas the value/number of the remaining length.
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_publish() {
        use super::*;
        let mut buffer: BytesMut = BytesMut::new();
        let topic_name: Bytes = Bytes::from("test_topic");
        let payload_value: Bytes = Bytes::from("test_payload");
        let retain_flag: bool = false;
        let publish: Publish = Publish::new(topic_name, payload_value, retain_flag);
        // test the write function of publish packet
        write(&publish, &mut buffer);

        // test the read function of publish packet and check the result of write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b00110000);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 24);
        let publish_msg: Publish = read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(publish_msg.topic, "test_topic");
        assert_eq!(publish_msg.payload, "test_payload");

        // test the display of publish packet
        println!("publish display: {}", publish);
    }
}
