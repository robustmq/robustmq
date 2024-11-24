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

use std::sync::Arc;

use dashmap::DashMap;
use log::error;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal_server::journal_engine::{
    GetClusterMetadataNode, GetShardMetadataReqShard, GetShardMetadataRespShard,
};

use crate::connection::ConnectionManager;
use crate::error::JournalClientError;
use crate::service::{get_cluster_metadata, get_shard_metadata};

#[derive(Default, Debug)]
pub struct MetadataCache {
    addrs: Vec<String>,
    shards: DashMap<String, GetShardMetadataRespShard>,
    nodes: DashMap<u64, GetClusterMetadataNode>,
    node_addr_node_id: DashMap<String, u64>,
}

impl MetadataCache {
    pub fn new(addrs: Vec<String>) -> Self {
        let shards = DashMap::with_capacity(8);
        let nodes = DashMap::with_capacity(2);
        let node_addr_node_id = DashMap::with_capacity(2);
        MetadataCache {
            addrs,
            shards,
            nodes,
            node_addr_node_id,
        }
    }

    pub fn get_tcp_addr_by_node_id(&self, node_id: u64) -> Option<String> {
        if let Some(node) = self.nodes.get(&node_id) {
            return Some(node.tcp_addr.clone());
        }
        None
    }

    pub fn add_shard(&self, shard: GetShardMetadataRespShard) {
        self.shards
            .insert(shard_name_iden(&shard.namespace, &shard.shard), shard);
    }

    pub fn remove_shard(&self, namespace: &str, shard: &str) {
        self.shards.remove(&shard_name_iden(namespace, shard));
    }

    pub fn add_node(&self, node: GetClusterMetadataNode) {
        self.nodes.insert(node.node_id, node);
    }

    pub fn all_node_ids(&self) -> Vec<u64> {
        let mut node_ids: Vec<u64> = self.nodes.iter().map(|raw| *raw.key()).collect();
        node_ids.sort();
        node_ids
    }

    pub fn all_shards(&self) -> Vec<GetShardMetadataRespShard> {
        let mut results = Vec::new();
        for raw in self.shards.iter() {
            results.push(GetShardMetadataRespShard {
                namespace: raw.namespace.clone(),
                shard: raw.shard.clone(),
                ..Default::default()
            });
        }
        results
    }
}

pub async fn load_shards_cache(
    metadata_cache: &Arc<MetadataCache>,
    connection_manager: &Arc<ConnectionManager>,
    namespace: String,
    shard_name: String,
) -> Result<(), JournalClientError> {
    let shards = vec![GetShardMetadataReqShard {
        namespace,
        shard_name,
    }];
    let resp = get_shard_metadata(connection_manager, shards).await?;
    for node in resp.shards {
        metadata_cache.add_shard(node);
    }
    Ok(())
}

pub async fn load_node_cache(
    metadata_cache: &Arc<MetadataCache>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), JournalClientError> {
    let resp = get_cluster_metadata(connection_manager).await?;
    for node in resp.nodes {
        metadata_cache.add_node(node);
    }
    Ok(())
}

pub async fn start_update_cache_thread(
    metadata_cache: Arc<MetadataCache>,
    connection_manager: Arc<ConnectionManager>,
) {
    tokio::spawn(async move {
        loop {
            // update node cache
            if let Err(e) = load_node_cache(&metadata_cache, &connection_manager).await {
                error!(
                    "Failed to update node cache periodically with error message :{}",
                    e
                );
            }

            // update shard cache
            for shard in metadata_cache.all_shards() {
                if let Err(e) = load_shards_cache(
                    &metadata_cache,
                    &connection_manager,
                    shard.namespace.clone(),
                    shard.shard.clone(),
                )
                .await
                {
                    if e.to_string().contains("does not exist") && e.to_string().contains("Shard") {
                        metadata_cache.remove_shard(&shard.namespace, &shard.shard);
                    } else {
                        error!(
                            "Failed to update shard cache periodically with error message :{}",
                            e
                        );
                    }
                }
            }
        }
    });
}
