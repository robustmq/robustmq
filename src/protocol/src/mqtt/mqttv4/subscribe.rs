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

fn len(subscribe: &Subscribe) -> usize {
    // length of pkid (variable header 2 bytes) + vec! [subscribed topic filter length]
    2 + subscribe.filters.iter().fold(0, |s, t| s + filter::len(t))
}

pub fn write(subscribe: &Subscribe, buffer: &mut BytesMut) -> Result<usize, MQTTProtocolError> {
    // write packet type of subscribe
    buffer.put_u8(0x82);

    // write remaining length
    let remaining_length = len(subscribe);
    let remaining_length_bytes = write_remaining_length(buffer, remaining_length)?;

    // write packet id
    buffer.put_u16(subscribe.packet_identifier);

    // write filters
    for f in subscribe.filters.iter() {
        filter::write(f, buffer);
    }

    Ok(1 + remaining_length_bytes + remaining_length)
}

pub fn read(fixed_header: FixedHeader, mut bytes: Bytes) -> Result<Subscribe, MQTTProtocolError> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    let pkid = read_u16(&mut bytes)?;
    let filters = filter::read(&mut bytes)?;

    match filters.len() {
        0 => Err(MQTTProtocolError::EmptySubscription),
        _ => Ok(Subscribe {
            packet_identifier: pkid,
            filters,
        }),
    }
}

mod filter {
    use super::*;

    pub fn len(filter: &Filter) -> usize {
        // filter length (2 bytes MSB + LSB)+ filter(subscribed topic name) +
        // options (QoS for 1 byte)
        2 + filter.path.len() + 1
    }

    pub fn read(bytes: &mut Bytes) -> Result<Vec<Filter>, MQTTProtocolError> {
        // variable header size 2 bytes (packet identifier)
        let mut filters = Vec::new();
        while bytes.has_remaining() {
            let path = read_mqtt_bytes(bytes)?;
            let path = std::str::from_utf8(&path)?.to_owned();
            let options = read_u8(bytes)?;
            let requested_qos = options & 0b0000_0011;

            filters.push(Filter {
                path,
                qos: qos(requested_qos).ok_or(MQTTProtocolError::InvalidQoS(requested_qos))?,
                // the following options only valid in mqtt v5 and will be ignored in mqtt v4
                no_local: false,
                preserve_retain: false,
                retain_handling: RetainHandling::OnEverySubscribe,
            });
        }

        Ok(filters)
    }

    pub fn write(filter: &Filter, buffer: &mut BytesMut) {
        let mut options = 0;
        options |= filter.qos as u8;

        write_mqtt_string(buffer, filter.path.as_str());
        buffer.put_u8(options);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_subscribe_filter() {
        use super::*;
        // test filter write function
        let mut buffer = BytesMut::new();
        let topic_filter = Filter {
            path: "test_sub_topic".to_string(),
            qos: QoS::AtLeastOnce,
            no_local: false, // invalid in mqtt v4, ignore here with default value
            preserve_retain: false, // invalid in mqtt v4, ignore here with default value
            retain_handling: RetainHandling::OnEverySubscribe, // invalid in mqtt v4, ignore here with default value
        };
        filter::write(&topic_filter, &mut buffer);

        // test filter read function and verify the result of write function

        let topic_filters_read: Vec<Filter> =
            filter::read(&mut buffer.copy_to_bytes(buffer.len())).unwrap();

        for i in &topic_filters_read {
            // only check topic path and qos which are valid in mqtt v4
            assert_eq!(i.path, "test_sub_topic");
            assert_eq!(i.qos, QoS::AtLeastOnce);
        }
    }

    #[test]
    fn test_subscribe() {
        use super::*;
        // build filter
        let mut buffer = BytesMut::new();
        let topic_filter = Filter {
            path: "test_sub_topic".to_string(),
            qos: QoS::AtLeastOnce,
            no_local: false, // invalid in mqtt v4, ignore here with default value
            preserve_retain: false, // invalid in mqtt v4, ignore here with default value
            retain_handling: RetainHandling::OnEverySubscribe, // invalid in mqtt v4, ignore here with default value
        };

        let vec = vec![topic_filter];

        // build subscribe
        let subscribe: Subscribe = Subscribe {
            packet_identifier: 5u16,
            filters: vec,
        };

        // test write function of subscribe
        write(&subscribe, &mut buffer).unwrap();

        // test read function of subscribe and verify the result of write function
        let fixed_header: FixedHeader = parse_fixed_header(buffer.iter()).unwrap();
        assert_eq!(fixed_header.byte1, 0b1000_0010);
        assert_eq!(fixed_header.fixed_header_len, 2);
        assert_eq!(fixed_header.remaining_len, 19);
        let subscribe_read: Subscribe =
            read(fixed_header, buffer.copy_to_bytes(buffer.len())).unwrap();
        assert_eq!(subscribe_read.packet_identifier, 5u16);
        assert_eq!(subscribe_read.filters.len(), 1);
    }
}
