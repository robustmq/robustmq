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

use bytes::{BufMut, BytesMut};
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use protocol::codec::RobustMQCodec;
use protocol::codec::RobustMQCodecWrapper;
use quinn::{RecvStream, SendStream};
use tokio::io::AsyncReadExt;
use tokio_util::codec::{Decoder, Encoder};
use tracing::{debug, info};

// Write Stream
pub struct QuicFramedWriteStream {
    write_stream: SendStream,
    codec: RobustMQCodec,
}

impl QuicFramedWriteStream {
    pub fn new(write_stream: SendStream, codec: RobustMQCodec) -> Self {
        Self {
            write_stream,
            codec,
        }
    }

    pub async fn send(&mut self, packet: RobustMQCodecWrapper) -> ResultCommonError {
        let mut bytes_mut = BytesMut::new();
        self.codec.encode(packet, &mut bytes_mut)?;
        let data_length = bytes_mut.len();

        debug!(
            "send stream data len: {}, data: {:x?}",
            data_length, &bytes_mut
        );

        let mut buf = BytesMut::with_capacity(4 + data_length);
        buf.put_u32(data_length as u32);
        buf.put_slice(&bytes_mut);

        if !bytes_mut.is_empty() {
            self.write_stream.write_all(&buf).await?;
        }
        Ok(())
    }
}

// Read Stream
pub struct QuicFramedReadStream {
    read_stream: RecvStream,
    codec: RobustMQCodec,
}

impl QuicFramedReadStream {
    pub fn new(read_stream: RecvStream, codec: RobustMQCodec) -> Self {
        Self { read_stream, codec }
    }

    #[allow(clippy::result_large_err)]
    pub async fn receive(&mut self) -> Result<Option<RobustMQCodecWrapper>, CommonError> {
        let data_length = self.read_stream.read_u32().await? as usize;
        info!("Expected to read {} bytes from stream", data_length);

        let mut body = vec![0u8; data_length];
        let mut decode_bytes = BytesMut::with_capacity(data_length);

        self.read_stream.read_exact(&mut body).await?;
        decode_bytes.extend_from_slice(&body);

        debug!(
            "read stream data len: {}, data: {:?}",
            decode_bytes.len(),
            decode_bytes
        );

        if !decode_bytes.is_empty() {
            return self.codec.decode(&mut decode_bytes);
        }

        Ok(None)
    }
}
