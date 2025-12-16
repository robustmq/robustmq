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
    use crate::meta::common::{namespace, pc_addr, shard_name};
    use metadata_struct::journal::shard::JournalShardConfig;
    use protocol::meta::meta_service_journal::engine_service_client::EngineServiceClient;
    use protocol::meta::meta_service_journal::{
        CreateNextSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest,
        ListShardRequest,
    };

    #[tokio::test]
    async fn test_create_shard() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let config = JournalShardConfig {
            max_segment_size: 1024 * 1024 * 10,
            replica_num: 1,
        };

        let request = CreateShardRequest {
            namespace: namespace(),
            shard_name: shard_name(),
            shard_config: config.encode().unwrap(),
        };
        client
            .create_shard(tonic::Request::new(request))
            .await
            .unwrap();

        let request = ListShardRequest {
            namespace: namespace(),
            shard_name: shard_name(),
        };
        
        let reply = client.list_shard(request).await.unwrap();
        assert_eq!(reply.into_inner().shards.len(), 1);

        let request = DeleteShardRequest {
            namespace: namespace(),
            shard_name: shard_name(),
        };
        client
            .delete_shard(tonic::Request::new(request))
            .await
            .unwrap();

        let request = ListShardRequest {
            namespace: namespace(),
            shard_name: shard_name(),
        };
        let reply = client.list_shard(request).await.unwrap();
        assert_eq!(reply.into_inner().shards.len(), 0);
    }

    #[tokio::test]
    async fn test_create_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = CreateNextSegmentRequest {
            namespace: namespace(),
            shard_name: shard_name(),
        };
        match client
            .create_next_segment(tonic::Request::new(request))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("{e}");
            }
        }
    }

    #[tokio::test]
    async fn test_delete_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = DeleteSegmentRequest {
            namespace: namespace(),
            shard_name: shard_name(),
            segment_seq: 1,
        };
        match client.delete_segment(tonic::Request::new(request)).await {
            Ok(_) => {}
            Err(e) => {
                println!("{e}");
            }
        }
    }
}
