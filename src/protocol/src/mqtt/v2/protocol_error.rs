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

use crate::mqtt::v2::mqttv4::mqtt_protocol_error::MQTTProtocolError;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum ProtocolError {
    #[error("Unknown protocol")]
    UnknownProtocol,

    #[error("from MQTTProtocolError: {0}")]
    MQTTProtocolError(#[from] MQTTProtocolError),
}

#[cfg(test)]
mod protocol_error_tests {
    use super::*;

    #[test]
    fn protocol_error_unknown_protocol() {
        let error = ProtocolError::UnknownProtocol;
        assert_eq!(format!("{}", error), "Unknown protocol");
    }

    #[test]
    fn protocol_error_mqtt_protocol_error() {
        let mqtt_error = MQTTProtocolError::InvalidPacketType;
        let protocol_error: ProtocolError = mqtt_error.into();
        assert_eq!(
            format!("{}", protocol_error),
            "from MQTTProtocolError: Invalid packet type: reserved bits are forbidden to use"
        );
    }
}
