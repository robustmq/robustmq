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
    use broker_core::cache::NodeCacheManager;
    use common_config::broker::default_broker_config;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::meta::node::BrokerNode;
    use node_call::{NodeCallData, NodeCallManager};
    use prost::Message;
    use protocol::broker::broker_mqtt::GetQosDataByClientIdReply;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use tokio::time::{sleep, Duration};

    fn make_node(node_id: u64, grpc_addr: &str) -> BrokerNode {
        BrokerNode {
            node_id,
            grpc_addr: grpc_addr.to_string(),
            ..Default::default()
        }
    }

    /// Verifies that send_with_reply dispatches to all registered nodes and returns
    /// per-client-id scoped replies encoded as Bytes.
    /// Run with a real broker listening on 127.0.0.1:1228.
    #[tokio::test]
    async fn send_with_reply_get_qos_data() {
        let config = default_broker_config();
        let broker_cache = Arc::new(NodeCacheManager::new(config));

        // Register a single broker node (real server must be running).
        broker_cache.add_node(make_node(1, "127.0.0.1:1228"));

        let client_pool = Arc::new(ClientPool::new(3));
        let manager = Arc::new(NodeCallManager::new(client_pool, broker_cache));

        let (stop_send, _stop_rx) = broadcast::channel::<bool>(2);

        // Start the manager in the background.
        let manager_bg = manager.clone();
        let stop_bg = stop_send.clone();
        tokio::spawn(async move {
            manager_bg.start(stop_bg).await;
        });

        // Wait until the global sender is ready.
        for _ in 0..20 {
            if manager.is_ready().await {
                break;
            }
            sleep(Duration::from_millis(50)).await;
        }
        assert!(
            manager.is_ready().await,
            "NodeCallManager did not become ready"
        );

        // 10 requests using 6 distinct client IDs (with deliberate duplicates).
        let client_ids = [
            "client-a", "client-b", "client-c", "client-a", "client-d", "client-b", "client-e",
            "client-f", "client-c", "client-a",
        ];

        for client_id in client_ids {
            let result = manager
                .send_with_reply(NodeCallData::GetQosData(client_id.to_string()))
                .await;

            // The call must succeed (server may return empty data for unknown clients,
            // but the RPC itself should not error out).
            assert!(
                result.is_ok(),
                "send_with_reply failed for client_id={client_id}: {:?}",
                result.err()
            );

            let raw_replies = result.unwrap();
            // One reply per registered node (we have 1 node).
            assert_eq!(
                raw_replies.len(),
                1,
                "expected 1 reply per node for client_id={client_id}"
            );

            // Decode and verify data scoping.
            for bytes in &raw_replies {
                let reply = GetQosDataByClientIdReply::decode(bytes.clone())
                    .expect("failed to decode GetQosDataByClientIdReply");
                for record in &reply.data {
                    assert_eq!(
                        record.client_id, client_id,
                        "reply data scoping error: got client_id={} but requested {}",
                        record.client_id, client_id
                    );
                }
            }
        }

        // Signal stop.
        let _ = stop_send.send(true);
    }
}
