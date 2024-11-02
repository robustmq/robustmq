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

    use common_base::tools::unique_id;
    use dashmap::DashMap;
    use futures::{SinkExt, StreamExt};
    use journal_client::tool::resp_header_error;
    use protocol::journal_server::codec::{JournalEnginePacket, JournalServerCodec};
    use protocol::journal_server::journal_engine::{
        ApiKey, ApiVersion, CreateShardReq, CreateShardReqBody, GetActiveSegmentReq,
        GetActiveSegmentReqBody, GetActiveSegmentReqShard, GetClusterMetadataReq, ReadReq,
        ReadReqBody, ReadReqMessage, ReadReqMessageOffset, ReqHeader, WriteReq, WriteReqBody,
        WriteReqMessages, WriteReqSegmentMessages,
    };
    use tokio::net::TcpStream;
    use tokio::time::sleep;
    use tokio_util::codec::Framed;

    #[tokio::test]
    async fn base_rw_test() {
        let server_addr = "127.0.0.1:3110";

        // get cluster node metadata
        let socket = TcpStream::connect(server_addr).await.unwrap();

        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::GetClusterMetadataReq(GetClusterMetadataReq {
            header: Some(ReqHeader {
                api_key: ApiKey::GetClusterMetadata.into(),
                api_version: ApiVersion::V0.into(),
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        let server_nodes = DashMap::with_capacity(2);
        if let Some(Ok(resp)) = stream.next().await {
            if let JournalEnginePacket::GetClusterMetadataResp(data) = resp {
                println!("{:?}", data);
                assert!(resp_header_error(&data.header.unwrap()).is_ok());

                for node in data.body.unwrap().nodes {
                    server_nodes.insert(node.node_id, node);
                }
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }

        println!("{:?}", server_nodes);

        let socket = TcpStream::connect(server_addr).await.unwrap();
        let namespace = unique_id();
        let shard_name = "s1".to_string();

        // create_shard
        let replica_num = 1;

        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::CreateShardReq(CreateShardReq {
            header: Some(ReqHeader {
                api_key: ApiKey::CreateShard.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(CreateShardReqBody {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num,
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(Ok(resp)) = stream.next().await {
            if let JournalEnginePacket::CreateShardResp(data) = resp {
                println!("{:?}", data);
                assert!(resp_header_error(&data.header.unwrap()).is_ok());
                let body = data.body.unwrap();
                let active_segment = body.active_segment.unwrap();
                assert_eq!(active_segment.segment_no, 0);
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }

        // get active shard
        let mut segment_0_all_replicas = Vec::new();
        let socket = TcpStream::connect("127.0.0.1:3110").await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::GetActiveSegmentReq(GetActiveSegmentReq {
            header: Some(ReqHeader {
                api_key: ApiKey::GetActiveSegment.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(GetActiveSegmentReqBody {
                shards: vec![GetActiveSegmentReqShard {
                    namespace: namespace.clone(),
                    shard_name: shard_name.clone(),
                }],
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(Ok(resp)) = stream.next().await {
            if let JournalEnginePacket::GetActiveSegmentResp(data) = resp {
                println!("{:?}", data);
                assert!(resp_header_error(&data.header.unwrap()).is_ok());
                let body = data.body.unwrap();
                let active_segment = body.segments.first().unwrap();
                let segment_metadata = active_segment.active_segment.clone().unwrap();
                assert_eq!(active_segment.namespace, namespace);
                assert_eq!(active_segment.shard, shard_name);
                assert_eq!(segment_metadata.replicas.len(), 1);
                segment_0_all_replicas = segment_metadata.replicas;
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }

        // write data
        let key = "k1".to_string();
        let value = serde_json::to_vec("dsfaerwqrsf").unwrap();
        let tags = vec!["t1".to_string()];

        for rep in segment_0_all_replicas {
            let rep_addr = server_nodes.get(&rep).unwrap();
            println!("{:?}", rep_addr.tcp_addr);
            let socket = TcpStream::connect(&rep_addr.tcp_addr).await.unwrap();
            let mut stream = Framed::new(socket, JournalServerCodec::new());

            let req_packet = JournalEnginePacket::WriteReq(WriteReq {
                header: Some(ReqHeader {
                    api_key: ApiKey::Write.into(),
                    api_version: ApiVersion::V0.into(),
                }),
                body: Some(WriteReqBody {
                    data: vec![WriteReqSegmentMessages {
                        namespace: namespace.clone(),
                        shard_name: shard_name.clone(),
                        segment: 0,
                        messages: vec![WriteReqMessages {
                            key: key.clone(),
                            value: value.clone(),
                            tags: tags.clone(),
                        }],
                    }],
                }),
            });

            let _ = stream.send(req_packet.clone()).await;

            if let Some(Ok(resp)) = stream.next().await {
                if let JournalEnginePacket::WriteResp(data) = resp {
                    println!("{:?}", data);
                    assert!(resp_header_error(&data.header.unwrap()).is_ok());
                    let body = data.body.unwrap();
                    let frist_status = body.status.first().unwrap();
                    assert_eq!(frist_status.messages.first().unwrap().offset, 0);
                } else {
                    assert!(false);
                }
            } else {
                assert!(false);
            }
        }

        // read data
        let socket = TcpStream::connect(server_addr).await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        let req_packet = JournalEnginePacket::ReadReq(ReadReq {
            header: Some(ReqHeader {
                api_key: ApiKey::Read.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(ReadReqBody {
                messages: vec![ReadReqMessage {
                    namespace: namespace.clone(),
                    shard_name: shard_name.clone(),
                    segments: vec![ReadReqMessageOffset {
                        segment: 0,
                        offset: 0,
                    }],
                }],
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(Ok(resp)) = stream.next().await {
            println!("{:?}", resp);
            if let JournalEnginePacket::ReadResp(data) = resp {
                println!("{:?}", data);
                assert!(resp_header_error(&data.header.unwrap()).is_ok());
                let body = data.body.unwrap();
                let msg = body.messages.first().unwrap();
                let raw_msg = msg.messages.first().unwrap();
                assert_eq!(msg.namespace, namespace);
                assert_eq!(msg.shard_name, shard_name);
                assert_eq!(raw_msg.key, key);
                assert_eq!(raw_msg.value, value);
                assert_eq!(raw_msg.tags, tags);
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }
}
