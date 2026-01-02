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

    use broker_core::cache::BrokerCacheManager;
    use common_config::config::BrokerConfig;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::storage::{
        codec::StorageEnginePacket,
        protocol::{ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType},
    };
    use storage_engine::{
        clients::manager::ClientConnectionManager, core::cache::StorageCacheManager,
    };

    #[tokio::test]
    async fn client_conn_test() {
        let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
        let node_id = 1;
        broker_cache.add_node(BrokerNode {
            node_id,
            engine_addr: "127.0.0.1:1778".to_string(),
            ..Default::default()
        });
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let client = Arc::new(ClientConnectionManager::new(cache_manager.clone(), 2));

        let read_req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            "test-shard".to_string(),
            ReadType::Offset,
            ReadReqFilter::by_offset(0),
            ReadReqOptions::new(1024 * 1024, 10),
        )]));

        let res = client
            .read_send(node_id, StorageEnginePacket::ReadReq(read_req))
            .await;

        match &res {
            Ok(resp) => println!("Success: {:?}", resp),
            Err(e) => println!("Error: {:?}", e),
        }

        assert!(res.is_ok(), "Failed to connect: {:?}", res);
    }
}
