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
    use common_base::tools::now_second;
    use futures::{SinkExt, StreamExt};
    use protocol::journal_server::codec::{JournalEnginePacket, JournalServerCodec};
    use protocol::journal_server::journal_engine::{
        ApiKey, ApiVersion, CreateShardReq, CreateShardReqBody, DeleteShardReq, DeleteShardReqBody,
        FetchOffsetReq, FetchOffsetReqBody, GetClusterMetadataReq, GetShardMetadataReq,
        GetShardMetadataReqBody, GetShardMetadataReqShard, ReadReq, ReadReqBody, ReadReqFilter,
        ReadReqMessage, ReadType, ReqHeader, WriteReq, WriteReqBody, WriteReqMessages,
        WriteReqSegmentMessages,
    };
    use tokio::net::TcpStream;
    use tokio_util::codec::Framed;

    use crate::journal_client::common::journal_tcp_addr;

    #[tokio::test]
    async fn get_cluster_metadata_base_test() {
        let socket = TcpStream::connect(&journal_tcp_addr()).await.unwrap();

        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::GetClusterMetadataReq(GetClusterMetadataReq {
            header: Some(ReqHeader {
                api_key: ApiKey::GetClusterMetadata.into(),
                api_version: ApiVersion::V0.into(),
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            let resp = data.unwrap();
            println!("{:?}", resp);
        }
    }

    #[tokio::test]
    async fn create_shard_test() {
        let socket = TcpStream::connect(&journal_tcp_addr()).await.unwrap();

        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::CreateShardReq(CreateShardReq {
            header: Some(ReqHeader {
                api_key: ApiKey::CreateShard.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(CreateShardReqBody {
                namespace: "b1".to_string(),
                shard_name: "s1".to_string(),
                replica_num: 1,
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            let resp = data.unwrap();
            println!("{:?}", resp);
        }
    }

    #[tokio::test]
    async fn get_shard_metadata_test() {
        let socket = TcpStream::connect(&journal_tcp_addr()).await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let shards = vec![GetShardMetadataReqShard {
            namespace: "b1".to_string(),
            shard_name: "s1".to_string(),
        }];

        let req_packet = JournalEnginePacket::GetShardMetadataReq(GetShardMetadataReq {
            header: Some(ReqHeader {
                api_key: ApiKey::GetShardMetadata.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(GetShardMetadataReqBody { shards }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            let resp = data.unwrap();
            println!("{:?}", resp);
        }
    }

    #[tokio::test]
    async fn delete_shard_test() {
        let socket = TcpStream::connect(&journal_tcp_addr()).await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::DeleteShardReq(DeleteShardReq {
            header: Some(ReqHeader {
                api_key: ApiKey::DeleteShard.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(DeleteShardReqBody {
                namespace: "b1".to_string(),
                shard_name: "s1".to_string(),
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            let resp = data.unwrap();
            println!("{:?}", resp);
        }
    }

    #[tokio::test]
    async fn write_base_test() {
        let socket = TcpStream::connect(&journal_tcp_addr()).await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::WriteReq(WriteReq {
            header: Some(ReqHeader {
                api_key: ApiKey::Write.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(WriteReqBody {
                data: vec![WriteReqSegmentMessages {
                    namespace: "b1".to_string(),
                    shard_name: "s1".to_string(),
                    segment: 0,
                    messages: vec![WriteReqMessages {
                        pkid: 1,
                        key: "k1".to_string(),
                        value: serde_json::to_vec(&now_second().to_string()).unwrap(),
                        tags: vec!["t1".to_string()],
                    }],
                }],
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            let resp = data.unwrap();
            println!("{:?}", resp);
        }
    }

    #[tokio::test]
    async fn read_base_test() {
        let socket = TcpStream::connect(&journal_tcp_addr()).await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::ReadReq(ReadReq {
            header: Some(ReqHeader {
                api_key: ApiKey::Read.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(ReadReqBody {
                messages: vec![ReadReqMessage {
                    namespace: "b1".to_string(),
                    shard_name: "s1".to_string(),
                    segment: 0,
                    ready_type: ReadType::Offset.into(),
                    filter: Some(ReadReqFilter {
                        offset: 0,
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            let resp = data.unwrap();
            println!("{:?}", resp);
        }
    }

    #[tokio::test]
    async fn fetch_offset_test() {
        let socket = TcpStream::connect(&journal_tcp_addr()).await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::FetchOffsetReq(FetchOffsetReq {
            header: Some(ReqHeader {
                api_key: ApiKey::Read.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(FetchOffsetReqBody {
                group_name: "g1".to_string(),
                ..Default::default()
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            let resp = data.unwrap();
            println!("{:?}", resp);
        }
    }
}
