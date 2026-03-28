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

use amq_protocol::frame::AMQPFrame;

use crate::{
    amqp::codec::AmqpCodec,
    kafka::{codec::KafkaCodec, packet::KafkaPacketWrapper},
    mqtt::{
        codec::{MqttCodec, MqttPacketWrapper},
        common::mqtt_packet_to_string,
    },
    nats::{codec::NatsCodec, packet::NatsPacket},
    robust::RobustMQProtocol,
    storage::codec::{StorageEngineCodec, StorageEnginePacket},
};
use bytes::BytesMut;
use common_base::error::common::CommonError;
use std::fmt;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub enum RobustMQCodecWrapper {
    StorageEngine(StorageEnginePacket),
    KAFKA(KafkaPacketWrapper),
    MQTT(MqttPacketWrapper),
    AMQP(AMQPFrame),
    NATS(NatsPacket),
}

impl fmt::Display for RobustMQCodecWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RobustMQCodecWrapper::MQTT(wrapper) => {
                write!(
                    f,
                    "MQTT(v{}, {})",
                    wrapper.protocol_version,
                    mqtt_packet_to_string(&wrapper.packet)
                )
            }
            RobustMQCodecWrapper::KAFKA(wrapper) => {
                write!(f, "KAFKA(api_version={})", wrapper.api_version)
            }
            RobustMQCodecWrapper::StorageEngine(packet) => {
                write!(f, "StorageEngine({})", packet)
            }
            RobustMQCodecWrapper::AMQP(frame) => {
                write!(f, "AMQP({frame})")
            }
            RobustMQCodecWrapper::NATS(pkt) => {
                write!(f, "NATS({pkt:?})")
            }
        }
    }
}

#[derive(Clone)]
pub struct RobustMQCodec {
    pub protocol: Option<RobustMQProtocol>,
    pub mqtt_codec: MqttCodec,
    pub kafka_codec: KafkaCodec,
    pub amqp_codec: AmqpCodec,
    pub storage_engine_codec: StorageEngineCodec,
    pub nats_codec: NatsCodec,
}

impl RobustMQCodec {
    pub fn new() -> Self {
        RobustMQCodec {
            protocol: None,
            mqtt_codec: MqttCodec::new(None),
            kafka_codec: KafkaCodec::new(),
            amqp_codec: AmqpCodec::new(),
            storage_engine_codec: StorageEngineCodec::new(),
            nats_codec: NatsCodec::new(),
        }
    }

    /// Create a codec with the protocol pre-set, skipping probe on first packet.
    pub fn new_with_protocol(protocol: RobustMQProtocol) -> Self {
        RobustMQCodec {
            protocol: Some(protocol),
            ..Self::new()
        }
    }
}

impl Default for RobustMQCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl RobustMQCodec {
    #[allow(clippy::result_large_err)]
    pub fn decode_data(
        &mut self,
        stream: &mut BytesMut,
    ) -> Result<Option<RobustMQCodecWrapper>, CommonError> {
        match self.protocol.clone() {
            Some(RobustMQProtocol::MQTT3 | RobustMQProtocol::MQTT4 | RobustMQProtocol::MQTT5) => {
                if let Ok(Some(pkg)) = self.mqtt_codec.decode_data(stream) {
                    self.protocol = self
                        .mqtt_codec
                        .protocol_version
                        .map(RobustMQProtocol::from_u8)
                        .or(self.protocol.clone());
                    return Ok(Some(RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
                        protocol_version: self.mqtt_codec.protocol_version.unwrap_or(4),
                        packet: pkg,
                    })));
                }
            }
            Some(RobustMQProtocol::KAFKA) => {
                if let Ok(Some(pkg)) = self.kafka_codec.decode_data(stream) {
                    return Ok(Some(RobustMQCodecWrapper::KAFKA(pkg)));
                }
            }
            Some(RobustMQProtocol::AMQP) => {
                if let Ok(Some(pkg)) = self.amqp_codec.decode_data(stream) {
                    return Ok(Some(RobustMQCodecWrapper::AMQP(pkg)));
                }
            }
            Some(RobustMQProtocol::StorageEngine) => {
                if let Ok(Some(pkg)) = self.storage_engine_codec.decode_data(stream) {
                    return Ok(Some(RobustMQCodecWrapper::StorageEngine(pkg)));
                }
            }
            Some(RobustMQProtocol::NATS) => {
                use tokio_util::codec::Decoder;
                if let Ok(Some(pkt)) = self.nats_codec.decode(stream) {
                    return Ok(Some(RobustMQCodecWrapper::NATS(pkt)));
                }
            }
            None => {
                if let Ok(Some(pkg)) = self.mqtt_codec.decode_data(stream) {
                    self.protocol = self
                        .mqtt_codec
                        .protocol_version
                        .map(RobustMQProtocol::from_u8);
                    return Ok(Some(RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
                        protocol_version: self.mqtt_codec.protocol_version.unwrap_or(4),
                        packet: pkg,
                    })));
                }
            }
        }

        Ok(None)
    }

    #[allow(clippy::result_large_err)]
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
            RobustMQCodecWrapper::AMQP(frame) => {
                if let Err(e) = self.amqp_codec.encode_data(frame, buffer) {
                    return Err(CommonError::CommonError(e.to_string()));
                }
            }
            RobustMQCodecWrapper::NATS(pkt) => {
                use tokio_util::codec::Encoder;
                self.nats_codec
                    .encode(pkt, buffer)
                    .map_err(|e| CommonError::CommonError(e.to_string()))?;
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
