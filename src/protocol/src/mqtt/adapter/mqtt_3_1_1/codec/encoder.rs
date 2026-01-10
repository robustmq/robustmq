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

use crate::mqtt::adapter::mqtt_3_1_1::codec::Mqtt3_1_1Codec;
use crate::mqtt::adapter::mqtt_3_1_1::packet::Packet;
use bytes::BytesMut;
use common_base::error::mqtt_protocol_error::MQTTProtocolError;
use tokio_util::codec;

#[allow(dead_code)]
impl codec::Encoder<Packet> for Mqtt3_1_1Codec {
    type Error = MQTTProtocolError;

    fn encode(&mut self, _item: Packet, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
