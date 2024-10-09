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

mod common;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bytes::Bytes;
    use futures::{SinkExt, StreamExt};
    use protocol::mqtt::{
        common::{Connect, LastWill, Login, MQTTPacket},
        mqttv4::codec::Mqtt4Codec,
    };
    use tokio::{
        net::TcpStream,
        time::{sleep, Instant},
    };
    use tokio_util::codec::Framed;

    #[tokio::test]
    async fn mqtt4_keep_alive_test() {
        let socket = TcpStream::connect("127.0.0.1:1883").await.unwrap();
        let mut stream: Framed<TcpStream, Mqtt4Codec> = Framed::new(socket, Mqtt4Codec::new());

        // send connect package
        let packet = build_mqtt4_pg_connect();
        let _ = stream.send(packet).await;
        let now = Instant::now();
        loop {
            if let Some(data) = stream.next().await {
                match data {
                    Ok(da) => {
                        println!("success:{:?}", da);
                    }
                    Err(e) => {
                        if !e.to_string().contains("Insufficient number") {
                            break;
                        }
                        println!("error:{}", e.to_string());
                    }
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        let ts = now.elapsed().as_secs();
        println!("{}",ts);
        assert!(ts >= 9 && ts <= 12);
    }

    /// Build the connect content package for the mqtt4 protocol
    fn build_mqtt4_pg_connect() -> MQTTPacket {
        let client_id = String::from("test_client_id");
        let login = Some(Login {
            username: "admin".to_string(),
            password: "pwd123".to_string(),
        });
        let lastwill = Some(LastWill {
            topic: Bytes::from("topic1"),
            message: Bytes::from("connection content"),
            qos: protocol::mqtt::common::QoS::AtLeastOnce,
            retain: true,
        });

        let connect: Connect = Connect {
            keep_alive: 5, // 30 seconds
            client_id: client_id,
            clean_session: true,
        };
        return MQTTPacket::Connect(4, connect, None, lastwill, None, login);
    }
}
