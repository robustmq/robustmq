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

    use crate::mqtt::protocol::quic_server::common::build_client_endpoint;
    use quinn::VarInt;
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    #[ignore = "reason"]
    async fn quic_client_connect_test() {
        let client_endpoint = build_client_endpoint("0.0.0.0");

        let server_addr: SocketAddr = "udp::127.0.0.1:9083".parse().unwrap();

        let connection_result = timeout(Duration::from_secs(10), async {
            client_endpoint
                .connect(server_addr, "localhost")
                .unwrap()
                .await
        })
        .await;
        assert!(connection_result.is_ok());

        let connection = connection_result.unwrap().unwrap();
        connection.close(VarInt::from_u32(0), b"test completed");
        client_endpoint.wait_idle().await;

        let close_result = timeout(Duration::from_secs(10), connection.closed()).await;
        assert!(close_result.is_ok());
    }
}
