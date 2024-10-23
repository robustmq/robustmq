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
    use protocol::journal_server::codec::{JournalEnginePacket, JournalServerCodec};
    use protocol::journal_server::journal_engine::{
        ApiKey, ApiVersion, GetClusterMetadataReq, ReqHeader,
    };
    use tokio::net::TcpStream;
    use tokio_util::codec::Framed;

    #[tokio::test]
    #[ignore]
    async fn get_cluster_metadata_base_test() {
        let socket = TcpStream::connect("127.0.0.1:3110").await.unwrap();

        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::GetClusterMetadataReq(GetClusterMetadataReq {
            header: Some(ReqHeader {
                api_key: ApiKey::GetClusterMetadata.into(),
                api_version: ApiVersion::V0.into(),
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            match data {
                Ok(da) => {
                    // assert_eq!(da, resp_packet.packet)
                    println!("{:?}", da);
                }
                Err(e) => {
                    panic!("error: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn create_shard_test() {}

    #[tokio::test]
    async fn delete_shard_test() {}

    #[tokio::test]
    async fn get_active_segment_test() {}

    #[tokio::test]
    async fn write_base_test() {}

    #[tokio::test]
    async fn read_base_test() {}

    #[tokio::test]
    async fn offset_base_test() {}
}
