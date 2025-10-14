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
    use std::sync::Arc;

    use grpc_clients::journal::admin::call::{
        journal_admin_list_segment, journal_admin_list_shard,
    };
    use grpc_clients::pool::ClientPool;
    use protocol::journal::journal_admin::{ListSegmentRequest, ListShardRequest};

    use crate::journal::server::common::journal_grpc_addr;

    #[ignore]
    #[tokio::test]
    async fn list_shard_list() {
        let addr = journal_grpc_addr();
        let client_pool = Arc::new(ClientPool::new(10));
        let addrs = vec![addr];
        let request = ListShardRequest::default();
        let data = journal_admin_list_shard(&client_pool, &addrs, request)
            .await
            .unwrap();
        println!("{:?}", data.shards);
    }

    #[ignore]
    #[tokio::test]
    async fn list_segment_list() {
        let addr = journal_grpc_addr();
        let client_pool = Arc::new(ClientPool::new(10));
        let addrs = vec![addr];
        let request = ListSegmentRequest {
            namespace: "b1".to_string(),
            shard_name: "s1".to_string(),
            segment_no: -1,
        };
        let data = journal_admin_list_segment(&client_pool, &addrs, request)
            .await
            .unwrap();
        println!("{:?}", data.segments);
    }
}
