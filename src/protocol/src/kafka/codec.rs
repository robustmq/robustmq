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

use bytes::BytesMut;
use tokio_util::codec;

use crate::mqtt::{codec::MqttPacketWrapper, common::MqttPacket};

#[derive(Clone, Debug)]
pub struct KafkaCodec {}

impl KafkaCodec {
    pub fn new() -> KafkaCodec {
        KafkaCodec {}
    }
}

impl Default for KafkaCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl KafkaCodec {
    pub fn decode_data(
        &mut self,
        stream: &mut BytesMut,
    ) -> Result<Option<MqttPacket>, crate::mqtt::common::Error> {
        println!("111");
        Ok(None)
    }

    pub fn encode_data(
        &mut self,
        packet_wrapper: MqttPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), crate::mqtt::common::Error> {
        println!("22");
        Ok(())
    }
}

impl codec::Encoder<MqttPacketWrapper> for KafkaCodec {
    type Error = crate::mqtt::common::Error;
    fn encode(
        &mut self,
        packet_wrapper: MqttPacketWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        self.encode_data(packet_wrapper, buffer)
    }
}

impl codec::Decoder for KafkaCodec {
    type Item = MqttPacket;
    type Error = crate::mqtt::common::Error;
    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_data(stream)
    }
}
