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

use quinn::{RecvStream, SendStream};

pub trait StreamOperator {
    fn send_message(&self, message: String);
    fn receive_message(&self) -> String;
}

#[allow(dead_code)]
pub struct QuicStream {
    write_stream: SendStream,
    read_stream: RecvStream,
}

impl QuicStream {
    fn new(write_stream: SendStream, read_stream: RecvStream) -> QuicStream {
        QuicStream {
            write_stream,
            read_stream,
        }
    }
    pub fn create_stream(write_stream: SendStream, read_stream: RecvStream) -> QuicStream {
        QuicStream::new(write_stream, read_stream)
    }
}

impl StreamOperator for QuicStream {
    fn send_message(&self, _message: String) {
        // Implementation to send a message over the write stream.
    }

    fn receive_message(&self) -> String {
        // Implementation to read and return a message from the read stream.
        "".to_string()
    }
}
