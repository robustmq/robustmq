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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::quic::quic_common::set_up;
    use mqtt_broker::server::quic::stream_handler::{QuicStream, StreamOperator};

    #[tokio::test]
    async fn test_send_message() {
        let (server, mut client) = set_up().await;

        let client_addr = client.local_addr();
        let server_addr = server.local_addr();

        let connection = client.connect(server_addr, "localhost").await.unwrap();

        let (write_stream, recv_stream) = connection.open_bi().await.unwrap();

        let stream = QuicStream::create_stream(write_stream, recv_stream);

        stream.send_message("test".to_string()).await.unwrap();
    }
}
