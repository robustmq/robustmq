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

use crate::core::cache::MetaCacheManager;
use crate::core::error::MetaServiceError;
use crate::core::notify::{send_notify_by_delete_shard, send_notify_by_delete_topic};
use crate::core::shard::delete_shard_by_real;
use crate::raft::manager::MultiRaftManager;
use crate::storage::mqtt::topic::MqttTopicStorage;
use crate::storage::topic_delete::TopicDeleteStorage;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use grpc_clients::broker::common::call::broker_get_shard_segment_delete_status;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::storage::shard::EngineShard;
use node_call::NodeCallManager;
use protocol::broker::broker::{GetShardSegmentDeleteStatusRequest, ShardSegmentStatusItem};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};
use tracing::{info, warn};

const TOPIC_DELETE_INTERVAL_MS: u64 = 5 * 1000;
const TOPIC_DELETE_CONCURRENCY: usize = 20;

pub async fn start_topic_delete_thread(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    node_call_manager: Arc<NodeCallManager>,
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if let Err(e) = notify_delete_topics(
            &rocksdb_engine_handler,
            &node_call_manager,
            &raft_manager,
            &cache_manager,
            &client_pool,
        )
        .await
        {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, TOPIC_DELETE_INTERVAL_MS, &stop_send).await;
}

async fn notify_delete_topics(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    node_call_manager: &Arc<NodeCallManager>,
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<MetaCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), MetaServiceError> {
    let storage = TopicDeleteStorage::new(rocksdb_engine_handler.clone());
    let topics = storage.all()?;

    let semaphore = Arc::new(Semaphore::new(TOPIC_DELETE_CONCURRENCY));
    let mut handles = Vec::with_capacity(topics.len());

    for topic in topics {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let node_call_manager = node_call_manager.clone();
        let raft_manager = raft_manager.clone();
        let cache_manager = cache_manager.clone();
        let client_pool = client_pool.clone();
        let rocksdb_engine_handler = rocksdb_engine_handler.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;

            // Notify brokers to delete each shard associated with this topic.
            for shard_name in topic.storage_name_list.values() {
                let shard = EngineShard {
                    shard_name: shard_name.clone(),
                    ..Default::default()
                };
                if let Err(e) = send_notify_by_delete_shard(&node_call_manager, shard).await {
                    warn!(
                        "Failed to notify brokers to delete shard: tenant={}, topic_name={}, shard_name={}, error={}",
                        topic.tenant, topic.topic_name, shard_name, e
                    );
                }
            }

            // Notify brokers to delete the topic itself.
            if let Err(e) = send_notify_by_delete_topic(&node_call_manager, topic.clone()).await {
                warn!(
                    "Failed to notify brokers to delete topic: tenant={}, topic_name={}, topic_id={}, error={}",
                    topic.tenant, topic.topic_name, topic.topic_id, e
                );
            }

            // Poll until all shards are confirmed deleted or timeout (5 min).
            let all_shards_deleted =
                wait_shards_deleted(&topic, &cache_manager, &client_pool).await;

            // Remove Resource if all shards are confirmed deleted.
            if all_shards_deleted {
                clean_topic_resources(
                    &topic,
                    &rocksdb_engine_handler,
                    &cache_manager,
                    &raft_manager,
                )
                .await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

const SHARD_DELETE_POLL_INTERVAL_MS: u64 = 3_000;
const SHARD_DELETE_TIMEOUT_MS: u64 = 5 * 60 * 1000;

/// Polls `check_all_shards_deleted` every 3 s until all shards are confirmed deleted
/// or 5 minutes have elapsed. Returns false on timeout so the topic can be retried next round.
async fn wait_shards_deleted(
    topic: &Topic,
    cache_manager: &Arc<MetaCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> bool {
    let deadline =
        tokio::time::Instant::now() + tokio::time::Duration::from_millis(SHARD_DELETE_TIMEOUT_MS);

    loop {
        if check_all_shards_deleted(topic, cache_manager, client_pool).await {
            return true;
        }

        if tokio::time::Instant::now() >= deadline {
            warn!(
                "Timed out waiting for shards to be deleted: tenant={}, topic_name={}, topic_id={}. Will retry next round.",
                topic.tenant, topic.topic_name, topic.topic_id
            );
            return false;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(
            SHARD_DELETE_POLL_INTERVAL_MS,
        ))
        .await;
    }
}

/// Returns true only if every shard in the topic reports deleted on every broker node.
async fn check_all_shards_deleted(
    topic: &Topic,
    cache_manager: &Arc<MetaCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> bool {
    let broker_addrs: Vec<String> = cache_manager
        .node_list
        .iter()
        .map(|e| e.value().grpc_addr.clone())
        .collect();

    if broker_addrs.is_empty() {
        return false;
    }

    // Build one batch request containing all shards of this topic.
    let items: Vec<ShardSegmentStatusItem> = topic
        .storage_name_list
        .values()
        .map(|shard_name| ShardSegmentStatusItem {
            shard_name: shard_name.clone(),
            segment_seq: None,
        })
        .collect();

    let req = GetShardSegmentDeleteStatusRequest {
        items: items.clone(),
    };

    // Query each broker node individually — all shards on all nodes must confirm deleted.
    for addr in &broker_addrs {
        match broker_get_shard_segment_delete_status(client_pool, &[addr], req.clone()).await {
            Ok(reply) => {
                if reply.results.iter().any(|r| !r.deleted) {
                    return false;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to get shard delete status: topic={}, broker={}, error={}",
                    topic.topic_name, addr, e
                );
                return false;
            }
        }
    }

    true
}

/// Delete topic records from RocksDB and clean up shard metadata from cache/raft.
async fn clean_topic_resources(
    topic: &Topic,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    cache_manager: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
) {
    // Remove from TopicDeleteStorage.
    let delete_storage = TopicDeleteStorage::new(rocksdb_engine_handler.clone());
    if let Err(e) = delete_storage.delete(&topic.topic_id) {
        warn!(
            "Failed to delete topic from TopicDeleteStorage: topic_id={}, error={}",
            topic.topic_id, e
        );
    }

    // Remove from MqttTopicStorage.
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    if let Err(e) = topic_storage.delete(&topic.tenant, &topic.topic_name) {
        warn!(
            "Failed to delete topic from MqttTopicStorage: tenant={}, topic_name={}, error={}",
            topic.tenant, topic.topic_name, e
        );
    }

    // Clean up each shard's metadata from cache and raft.
    for shard_name in topic.storage_name_list.values() {
        if let Err(e) = delete_shard_by_real(cache_manager, raft_manager, shard_name).await {
            warn!(
                "Failed to delete shard metadata: topic={}, shard={}, error={}",
                topic.topic_name, shard_name, e
            );
        }
    }

    info!(
        "Topic deleted successfully: tenant={}, topic_name={}, topic_id={}",
        topic.tenant, topic.topic_name, topic.topic_id
    );
}
