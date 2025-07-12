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
    use futures::{SinkExt, StreamExt};
    use protocol::kafka::codec::KafkaCodec;
    use robustmq_test::mqtt_build_tool::build_connect::build_mqtt4_pg_connect;
    use tokio::net::TcpListener;
    use tokio::{io, net::TcpStream};
    use tokio_util::codec::{Framed, FramedRead, FramedWrite};

    #[tokio::test]
    #[ignore = "reason"]
    async fn kafka_server() {
        let ip = "127.0.0.1:9092";
        let listener = TcpListener::bind(ip).await.unwrap();
        println!("555");
        let (stream, _) = listener.accept().await.unwrap();
        println!("444");
        let (r_stream, w_stream) = io::split(stream);
        let codec = KafkaCodec::new();
        let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
        let write_frame_stream = FramedWrite::new(w_stream, codec.clone());
        loop {
            println!("xxxx1");
            let data = read_frame_stream.next().await;
            println!("Got: {data:?}");
        }
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn kafka_client() {
        TcpStream::connect("127.0.0.1:9092").await.unwrap();
    }
}
