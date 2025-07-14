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
    use std::time::Duration;

    use futures::StreamExt;
    use protocol::kafka::codec::KafkaCodec;
    use tokio::net::TcpListener;
    use tokio::time::sleep;
    use tokio::{io, net::TcpStream};
    use tokio_util::codec::{FramedRead, FramedWrite};

    #[tokio::test]
    #[ignore = "reason"]
    async fn kafka_server() {
        let ip = "127.0.0.1:9092";
        let listener = TcpListener::bind(ip).await.unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let (r_stream, w_stream) = io::split(stream);
        let codec = KafkaCodec::new();
        let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
        let _write_frame_stream = FramedWrite::new(w_stream, codec.clone());
        loop {
            let data = read_frame_stream.next().await;
            if let Some(pack) = data {
                println!("recv:{:?}", pack);
            } else {
                sleep(Duration::from_millis(10)).await;
            }
        }
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn kafka_client() {
        TcpStream::connect("127.0.0.1:9092").await.unwrap();
    }
}
