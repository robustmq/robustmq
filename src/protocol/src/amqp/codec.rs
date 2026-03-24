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

use amq_protocol::frame::{gen_frame, parse_frame, AMQPFrame};
use bytes::{Buf, BytesMut};
use common_base::error::common::CommonError;
use cookie_factory::gen_simple;

// AMQP 0.9.1 frame layout:
//   1 byte  frame type
//   2 bytes channel id
//   4 bytes payload size
//   N bytes payload
//   1 byte  frame-end (0xCE)
const FRAME_MIN_HEADER: usize = 7;

#[derive(Clone, Debug)]
pub struct AmqpCodec {}

impl AmqpCodec {
    pub fn new() -> Self {
        AmqpCodec {}
    }

    pub fn decode_data(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<AMQPFrame>, Box<CommonError>> {
        // Need at least the 7-byte header to know total frame size.
        // ProtocolHeader is also 8 bytes ("AMQP\0\0\x09\x01"), handle below via parse_frame.
        if src.len() < FRAME_MIN_HEADER {
            return Ok(None);
        }

        // AMQP ProtocolHeader starts with 'A' (0x41). For all other frames, check size.
        // For regular frames: total = 7 (header) + payload_size + 1 (frame-end).
        if src[0] != b'A' {
            let payload_size = u32::from_be_bytes([src[3], src[4], src[5], src[6]]) as usize;
            let total = FRAME_MIN_HEADER + payload_size + 1;
            if src.len() < total {
                return Ok(None);
            }
        }

        match parse_frame(src.as_ref()) {
            Ok((remaining, frame)) => {
                let consumed = src.len() - remaining.len();
                src.advance(consumed);
                Ok(Some(frame))
            }
            Err(nom::Err::Incomplete(_)) => Ok(None),
            Err(e) => Err(Box::new(CommonError::CommonError(format!(
                "AMQP frame parse error: {e}"
            )))),
        }
    }

    pub fn encode_data(
        &mut self,
        frame: AMQPFrame,
        dst: &mut BytesMut,
    ) -> Result<(), Box<CommonError>> {
        let buf = gen_simple(gen_frame(&frame), Vec::new()).map_err(|e| {
            Box::new(CommonError::CommonError(format!(
                "AMQP frame encode error: {e}"
            )))
        })?;
        dst.extend_from_slice(&buf);
        Ok(())
    }
}

impl Default for AmqpCodec {
    fn default() -> Self {
        Self::new()
    }
}
