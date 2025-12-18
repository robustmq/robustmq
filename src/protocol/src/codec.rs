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

use crate::{
    kafka::{codec::KafkaCodec, packet::KafkaPacketWrapper},
    mqtt::codec::{MqttCodec, MqttPacketWrapper},
    robust::RobustMQProtocol,
    storage::codec::{StorageEngineCodec, StorageEnginePacket},
};
use bytes::BytesMut;
use common_base::error::common::CommonError;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub enum RobustMQCodecWrapper {
    StorageEngine(StorageEnginePacket),
    KAFKA(KafkaPacketWrapper),
    MQTT(MqttPacketWrapper),
}

pub enum RobustMQCodecEnum {
    MQTT(MqttCodec),
    KAFKA(KafkaCodec),
    StorageEngine(StorageEngineCodec),
}

#[derive(Clone)]
pub struct RobustMQCodec {
    pub protocol: Option<RobustMQProtocol>,
    pub mqtt_codec: MqttCodec,
    pub kafka_codec: KafkaCodec,
    pub storage_engine_codec: StorageEngineCodec,
}

impl RobustMQCodec {
    pub fn new() -> Self {
        RobustMQCodec {
            protocol: None,
            mqtt_codec: MqttCodec::new(None),
            kafka_codec: KafkaCodec::new(),
            storage_engine_codec: StorageEngineCodec::new(),
        }
    }
}

impl Default for RobustMQCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl RobustMQCodec {
    pub fn decode_data(
        &mut self,
        stream: &mut BytesMut,
    ) -> Result<Option<RobustMQCodecWrapper>, CommonError> {
        if let Some(protoc) = self.protocol.clone() {
            if protoc.is_kafka() {
                let res = self.kafka_codec.decode_data(stream);
                if let Ok(Some(pkg)) = res {
                    self.protocol = Some(RobustMQProtocol::KAFKA);
                    return Ok(Some(RobustMQCodecWrapper::KAFKA(pkg)));
                }
            }

            if protoc.is_mqtt() {
                let res = self.mqtt_codec.decode_data(stream);
                if let Ok(Some(pkg)) = res {
                    if self.protocol.is_none() {
                        self.protocol = self
                            .mqtt_codec
                            .protocol_version
                            .map(RobustMQProtocol::from_u8);
                    }
                    return Ok(Some(RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
                        protocol_version: self.mqtt_codec.protocol_version.unwrap_or(4),
                        packet: pkg,
                    })));
                }
            }
        } else {
            // try decode mqtt
            let res = self.mqtt_codec.decode_data(stream);
            if let Ok(Some(pkg)) = res {
                if self.protocol.is_none() {
                    self.protocol = self
                        .mqtt_codec
                        .protocol_version
                        .map(RobustMQProtocol::from_u8);
                }
                return Ok(Some(RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
                    protocol_version: self.mqtt_codec.protocol_version.unwrap_or(4),
                    packet: pkg,
                })));
            }

            // try decode kafka
            let res = self.kafka_codec.decode_data(stream);
            if let Ok(Some(pkg)) = res {
                self.protocol = Some(RobustMQProtocol::KAFKA);
                return Ok(Some(RobustMQCodecWrapper::KAFKA(pkg)));
            }
        }

        Ok(None)
    }

    pub fn encode_data(
        &mut self,
        packet_wrapper: RobustMQCodecWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), CommonError> {
        match packet_wrapper {
            RobustMQCodecWrapper::MQTT(wrapper) => {
                self.mqtt_codec.encode_data(wrapper, buffer)?;
            }
            RobustMQCodecWrapper::KAFKA(wrapper) => {
                self.kafka_codec.encode_data(wrapper, buffer)?;
            }
            RobustMQCodecWrapper::StorageEngine(wrapper) => {
                if let Err(e) = self.storage_engine_codec.encode_data(wrapper, buffer) {
                    return Err(CommonError::CommonError(e.to_string()));
                }
            }
        }

        Ok(())
    }
}

impl Decoder for RobustMQCodec {
    type Item = RobustMQCodecWrapper;
    type Error = CommonError;

    fn decode(&mut self, stream: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_data(stream)
    }
}

impl Encoder<RobustMQCodecWrapper> for RobustMQCodec {
    type Error = CommonError;
    fn encode(
        &mut self,
        packet_wrapper: RobustMQCodecWrapper,
        buffer: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        self.encode_data(packet_wrapper, buffer)?;
        Ok(())
    }
}
