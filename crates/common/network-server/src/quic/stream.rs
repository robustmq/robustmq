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
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use protocol::codec::RobustMQCodec;
use protocol::codec::RobustMQCodecWrapper;
use quinn::{RecvStream, SendStream};
use tokio_util::codec::{Decoder, Encoder};

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

        if !bytes_mut.is_empty() {
            self.write_stream.write_all(bytes_mut.as_mut()).await?;
            self.write_stream.finish()?;
            self.write_stream.stopped().await?;
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
        let mut decode_bytes = BytesMut::with_capacity(0);
        let vec = self.read_stream.read_to_end(1024).await?;
        decode_bytes.extend(vec);

        if !decode_bytes.is_empty() {
            return self.codec.decode(&mut decode_bytes);
        }

        Ok(None)
    }
}
