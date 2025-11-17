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

use crate::protocol::byte_wrapper::byte_operations::ByteOperations;
use crate::protocol::mqtt::mqtt_protocol_error::MQTTProtocolError;
use crate::protocol::utils::utf::utf_8_handler;

pub enum VariableHeader {
    Connect {
        protocol_level: u8,
        user_name_flag: bool,
        password_flag: bool,
        will_retain: bool,
        will_qos: u8,
        will_flag: bool,
        clean_session: bool,
        keep_alive: u16,
    },
    ConnAck,
    Publish,
    PubAck,
    PubRec,
    PubRel,
    PubComp,
    Subscribe,
    SubAck,
    Unsubscribe,
    UnsubAck,
    PingReq,
    PingResp,
    Disconnect,
}

impl VariableHeader {
    pub(super) fn verify_protocol_name(
        bytes: &mut impl ByteOperations,
    ) -> Result<(), MQTTProtocolError> {
        let protocol_name = utf_8_handler::parse(bytes)?;
        if protocol_name != "MQTT" {
            return Err(MQTTProtocolError::ProtocolNameError(protocol_name));
        }
        Ok(())
    }
}


impl VariableHeader {}
#[cfg(test)]
mod variable_header_tests {
    use crate::protocol::mqtt::mqtt4::variable_header::VariableHeader;
    use crate::protocol::mqtt::mqtt_protocol_error::MQTTProtocolError;
    use crate::protocol::utils::utf::utf_8_handler::write_utf8_for_mqtt;
    use bytes::BytesMut;

    // todo connect
    #[test]
    fn connect_can_allowed_valid_protocol_name() {
        let mut bytes_mut = BytesMut::new();
        write_utf8_for_mqtt(&mut bytes_mut, "MQTT").unwrap();
        VariableHeader::verify_protocol_name(&mut bytes_mut).unwrap();
    }
    #[test]
    fn connect_can_not_allowed_invalid_protocol_name() {
        let mut bytes_mut = BytesMut::new();
        let invalid_name = "hello";
        write_utf8_for_mqtt(&mut bytes_mut, invalid_name).unwrap();
        let result = VariableHeader::verify_protocol_name(&mut bytes_mut);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(MQTTProtocolError::ProtocolNameError(invalid_name))
        ));
    }
    // todo connect contain protocol level
    // todo connect contain connect flags
    // todo connect keep alive
    // todo connack
    // todo publish
    // todo puback
    // todo pubrec
    // todo pubrel
    // todo pubcomp
    // todo subscribe
    // todo suback
    // todo unsubscribe
    // todo unsuback
    // todo pingreq
    // todo pingresp
    // todo disconnect
}
