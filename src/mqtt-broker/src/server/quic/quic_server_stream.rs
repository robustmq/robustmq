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

use crate::handler::error::MqttBrokerError;
use bytes::BytesMut;
use common_base::error::common::CommonError::CommonError;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{Error, MqttPacket};
use quinn::{RecvStream, SendStream, WriteError};
use tokio_util::codec::{Decoder, Encoder};

pub struct QuicFramedWriteStream {
    write_stream: SendStream,
    codec: MqttCodec,
}

impl QuicFramedWriteStream {
    pub fn new(write_stream: SendStream, codec: MqttCodec) -> Self {
        Self {
            write_stream,
            codec,
        }
    }

    pub async fn send(&mut self, packet: MqttPacketWrapper) -> Result<(), MqttBrokerError> {
        let mut bytes_mut = BytesMut::new();

        if let Err(e) = self.codec.encode(packet, &mut bytes_mut) {
            return Err(MqttBrokerError::from(CommonError(format!(
                "encode packet failed: {}",
                e
            ))));
        }

        if bytes_mut.is_empty() {
            return Err(MqttBrokerError::from(CommonError(
                "encode packet failed: the packet is empty".to_string(),
            )));
        }

        if let Err(e) = self.write_stream.write_all(bytes_mut.as_mut()).await {
            return Err(MqttBrokerError::from(CommonError(format!(
                "write packet failed: {}",
                e
            ))));
        }

        if let Err(e) = self.write_stream.finish() {
            return Err(MqttBrokerError::from(CommonError(format!(
                "write packet failed: {}",
                e
            ))));
        }

        if let Err(e) = self.write_stream.stopped().await {
            return Err(MqttBrokerError::from(CommonError(format!(
                "write packet failed: {}",
                e
            ))));
        }

        Ok(())
    }
}

struct QuicFramedReadStream {
    read_stream: RecvStream,
    codec: MqttCodec,
}

impl QuicFramedReadStream {
    pub fn new(read_stream: RecvStream, codec: MqttCodec) -> Self {
        Self { read_stream, codec }
    }

    pub async fn receive(&mut self) -> Result<MqttPacket, MqttBrokerError> {
        let mut decode_bytes = BytesMut::with_capacity(0);
        let vec = self.read_stream.read_to_end(1024).await.map_err(|e| {
            MqttBrokerError::from(CommonError(format!(
                "read packet failed: {}",
                e.to_string()
            )))
        })?;
        decode_bytes.extend(vec);
        let packet = self.codec.decode(&mut decode_bytes).map_err(|e| {
            MqttBrokerError::from(CommonError(format!(
                "decode packet failed: {}",
                e.to_string()
            )))
        })?;
        Ok(packet.unwrap())
    }
}
