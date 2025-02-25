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

use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::mqtt::common::{ConnAck, ConnAckProperties, ConnectReturnCode, MqttPacket};

/// Build the connect content package for the mqtt5 protocol
pub fn build_mqtt5_pg_connect_ack_wrapper() -> MqttPacketWrapper {
    let ack: ConnAck = ConnAck {
        session_present: true,
        code: ConnectReturnCode::Success,
    };
    let properties = ConnAckProperties {
        max_qos: Some(10u8),
        ..Default::default()
    };
    MqttPacketWrapper {
        protocol_version: 5,
        packet: MqttPacket::ConnAck(ack, Some(properties)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use protocol::mqtt::codec::MqttCodec;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_build_mqtt5_pg_connect_ack_wrapper() {
        let wrapper = build_mqtt5_pg_connect_ack_wrapper();
        assert_eq!(wrapper.protocol_version, 5);
    }

    #[test]
    fn test_decode_mqtt5_pg_connect_ack() {
        let wrapper = build_mqtt5_pg_connect_ack_wrapper();
        let mut codec = MqttCodec::new(Some(5));
        let mut bytes_mut = BytesMut::with_capacity(0);
        codec.encode(wrapper, &mut bytes_mut).unwrap();

        let packet = codec.decode(&mut bytes_mut).unwrap();
        assert!(packet.is_some());
    }
}
