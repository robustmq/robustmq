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

    use common_base::tools::{now_second, unique_id};
    use dashmap::DashMap;
    use futures::{SinkExt, StreamExt};
    use journal_client::tool::resp_header_error;
    use protocol::journal::codec::{JournalEnginePacket, JournalServerCodec};
    use protocol::journal::journal_engine::{
        ApiKey, ApiVersion, CreateShardReq, CreateShardReqBody, FetchOffsetReq, FetchOffsetReqBody,
        FetchOffsetShard, GetClusterMetadataReq, GetShardMetadataReq, GetShardMetadataReqBody,
        GetShardMetadataReqShard, ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadType,
        ReqHeader, WriteReq, WriteReqBody, WriteReqMessages, WriteReqSegmentMessages,
    };
    use tokio::net::TcpStream;
    use tokio::time::sleep;
    use tokio_util::codec::Framed;

    use crate::journal::client::common::journal_tcp_addr;

    #[ignore]
    #[tokio::test]
    async fn base_rw_test() {
        let server_addr = journal_tcp_addr();

        let socket = TcpStream::connect(server_addr).await.unwrap();
        let mut stream = Framed::new(socket, JournalServerCodec::new());

        // get cluster node metadata
        let req_packet = JournalEnginePacket::GetClusterMetadataReq(GetClusterMetadataReq {
            header: Some(ReqHeader {
                api_key: ApiKey::GetClusterMetadata.into(),
                api_version: ApiVersion::V0.into(),
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        let server_nodes = DashMap::with_capacity(2);
        if let Some(Ok(JournalEnginePacket::GetClusterMetadataResp(data))) = stream.next().await {
            println!("{data:?}");
            assert!(resp_header_error(&data.header, req_packet).is_ok());
            for node in data.body.unwrap().nodes {
                server_nodes.insert(node.node_id, node);
            }
        } else {
            panic!()
        }

        println!("{server_nodes:?}");

        let namespace = unique_id();
        let shard_name = "s1".to_string();

        // create_shard
        let replica_num = 1;
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

        if let Some(Ok(JournalEnginePacket::CreateShardResp(data))) = stream.next().await {
            println!("{data:?}");
            assert!(resp_header_error(&data.header, req_packet).is_ok());
        } else {
            panic!();
        }
        sleep(Duration::from_secs(3)).await;

        // get shard metadata
        let req_packet = JournalEnginePacket::GetShardMetadataReq(GetShardMetadataReq {
            header: Some(ReqHeader {
                api_key: ApiKey::GetShardMetadata.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(GetShardMetadataReqBody {
                shards: vec![GetShardMetadataReqShard {
                    namespace: namespace.clone(),
                    shard_name: shard_name.clone(),
                }],
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        let segment_0_all_replicas = if let Some(Ok(JournalEnginePacket::GetShardMetadataResp(
            data,
        ))) = stream.next().await
        {
            println!("{data:?}");
            assert!(resp_header_error(&data.header, req_packet).is_ok());
            let body = data.body.unwrap();
            let shards = body.shards.first().unwrap();
            assert_eq!(shards.namespace, namespace);
            assert_eq!(shards.shard, shard_name);
            assert_eq!(shards.segments.len(), 1);
            let segment_metadata = shards.segments.first().unwrap();
            assert_eq!(segment_metadata.replicas.len(), 1);
            assert_eq!(segment_metadata.leader, 1);
            assert_eq!(segment_metadata.segment_no, 0);
            segment_metadata.replicas.clone()
        } else {
            panic!();
        };

        // write data
        let key = "k1".to_string();
        let value = serde_json::to_vec("dsfaerwqrsf").unwrap();
        let tags = vec!["t1".to_string()];

        let search_time = now_second();
        for rep in segment_0_all_replicas {
            let rep_addr = server_nodes.get(&rep).unwrap();
            println!("{:?}", rep_addr.tcp_addr);
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
                            pkid: 1,
                            key: key.clone(),
                            value: value.clone(),
                            tags: tags.clone(),
                        }],
                    }],
                }),
            });

            let _ = stream.send(req_packet.clone()).await;

            if let Some(Ok(JournalEnginePacket::WriteResp(data))) = stream.next().await {
                println!("{data:?}");
                assert!(resp_header_error(&data.header, req_packet).is_ok());
                let body = data.body.unwrap();
                let first_status = body.status.first().unwrap();
                assert_eq!(first_status.messages.first().unwrap().offset, 0);
            } else {
                panic!();
            }
        }

        // read data
        let req_packet = JournalEnginePacket::ReadReq(ReadReq {
            header: Some(ReqHeader {
                api_key: ApiKey::Read.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(ReadReqBody {
                messages: vec![ReadReqMessage {
                    namespace: namespace.to_string(),
                    shard_name: shard_name.to_string(),
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

        if let Some(Ok(resp)) = stream.next().await {
            println!("{resp:?}");
            if let JournalEnginePacket::ReadResp(data) = resp {
                assert!(resp_header_error(&data.header, req_packet).is_ok());
                let body = data.body.unwrap();
                let msg = body.messages.first().unwrap();
                let raw_msg = msg.messages.first().unwrap();
                assert_eq!(msg.namespace, namespace);
                assert_eq!(msg.shard_name, shard_name);
                assert_eq!(raw_msg.key, key);
                assert_eq!(raw_msg.value, value);
                assert_eq!(raw_msg.tags, tags);
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        // fetch Offset
        let req_packet = JournalEnginePacket::FetchOffsetReq(FetchOffsetReq {
            header: Some(ReqHeader {
                api_key: ApiKey::FetchOffset.into(),
                api_version: ApiVersion::V0.into(),
            }),
            body: Some(FetchOffsetReqBody {
                shards: vec![FetchOffsetShard {
                    namespace: namespace.to_string(),
                    shard_name: shard_name.to_string(),
                    segment_no: 0,
                    timestamp: search_time,
                }],
            }),
        });

        let _ = stream.send(req_packet.clone()).await;

        if let Some(Ok(resp)) = stream.next().await {
            println!("{resp:?}");
        } else {
            panic!();
        }
    }
}
