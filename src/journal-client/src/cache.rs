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

use crate::connection::ConnectionManager;
use crate::error::JournalClientError;
use crate::service::{get_cluster_metadata, get_shard_metadata};
use dashmap::DashMap;
use metadata_struct::journal::segment::segment_name;
use protocol::storage::journal_engine::{
    ClientSegmentMetadata, GetClusterMetadataNode, GetShardMetadataReqShard,
    GetShardMetadataRespShard,
};
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;
use tracing::error;

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

    pub fn get_rand_addr(&self) -> String {
        let mut rng = rand::thread_rng();
        self.addrs[rng.gen_range(0..self.addrs.len())].clone()
    }

    pub fn get_shard(&self, shard: &str) -> Option<GetShardMetadataRespShard> {
        if let Some(shard) = self.shards.get(shard) {
            return Some(shard.clone());
        }
        None
    }
    pub fn add_shard(&self, shard: GetShardMetadataRespShard) {
        self.shards.insert(shard.shard.clone(), shard);
    }

    pub fn get_active_segment(&self, shard: &str) -> Option<u32> {
        if let Some(shard) = self.shards.get(shard) {
            return Some(shard.active_segment as u32);
        }
        None
    }

    pub fn get_segment_leader(&self, shard: &str) -> Option<u32> {
        if let Some(shard) = self.shards.get(shard) {
            return Some(shard.active_segment_leader as u32);
        }
        None
    }

    pub fn get_leader_by_segment(&self, shard: &str, segment: u32) -> Option<u64> {
        if let Some(shard) = self.shards.get(shard) {
            for meta in shard.segments.iter() {
                if meta.segment_no == segment {
                    return Some(meta.leader);
                }
            }
        }
        None
    }

    pub fn remove_shard(&self, shard: &str) {
        self.shards.remove(shard);
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
                shard: raw.shard.clone(),
                ..Default::default()
            });
        }
        results
    }

    pub fn all_metadata(&self) -> (Vec<GetShardMetadataRespShard>, Vec<GetClusterMetadataNode>) {
        let shards = self.shards.iter().map(|raw| raw.clone()).collect();
        let nodes = self.nodes.iter().map(|raw| raw.clone()).collect();
        (shards, nodes)
    }
}

pub async fn load_shards_cache(
    metadata_cache: &Arc<MetadataCache>,
    connection_manager: &Arc<ConnectionManager>,
    shard_name: &str,
) -> Result<(), JournalClientError> {
    let shards = vec![GetShardMetadataReqShard {
        shard_name: shard_name.to_owned(),
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
    mut stop_recv: Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Err(flag) = val {

                    }
                },
                val = update_cache(metadata_cache.clone(),connection_manager.clone())=>{
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }
    });
}

async fn update_cache(
    metadata_cache: Arc<MetadataCache>,
    connection_manager: Arc<ConnectionManager>,
) {
    // update node cache
    if let Err(e) = load_node_cache(&metadata_cache, &connection_manager).await {
        error!(
            "Failed to update node cache periodically with error message :{}",
            e
        );
    }

    // update shard cache
    for shard in metadata_cache.all_shards() {
        if let Err(e) = load_shards_cache(&metadata_cache, &connection_manager, &shard.shard).await
        {
            if e.to_string().contains("does not exist") && e.to_string().contains("Shard") {
                metadata_cache.remove_shard(&shard.shard);
            } else {
                error!(
                    "Failed to update shard cache periodically with error message :{}",
                    e
                );
            }
        }
    }
}

pub async fn get_active_segment(
    metadata_cache: &Arc<MetadataCache>,
    connection_manager: &Arc<ConnectionManager>,
    shard_name: &str,
) -> u32 {
    loop {
        let active_segment = if let Some(segment) = metadata_cache.get_active_segment(shard_name) {
            segment
        } else {
            if let Err(e) = load_shards_cache(metadata_cache, connection_manager, shard_name).await
            {
                error!(
                    "Loading Shard {} Metadata info failed, error message :{}",
                    shard_name, e
                );
            }
            error!(
                "{}",
                JournalClientError::NotActiveSegment(shard_name.to_string()).to_string()
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        };
        return active_segment;
    }
}

pub async fn get_segment_leader(
    metadata_cache: &Arc<MetadataCache>,
    connection_manager: &Arc<ConnectionManager>,
    shard_name: &str,
) -> u64 {
    let active_segment = get_active_segment(metadata_cache, connection_manager, shard_name).await;

    loop {
        let leader = if let Some(leader) = metadata_cache.get_segment_leader(shard_name) {
            leader
        } else {
            if let Err(e) = load_shards_cache(metadata_cache, connection_manager, shard_name).await
            {
                error!(
                    "Loading Shard {} Metadata info failed, error message :{}",
                    shard_name, e
                );
            }
            error!(
                "{}",
                JournalClientError::NotLeader(segment_name(shard_name, active_segment)).to_string()
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        };
        return leader.into();
    }
}

pub async fn get_metadata_by_shard(
    metadata_cache: &Arc<MetadataCache>,
    connection_manager: &Arc<ConnectionManager>,
    shard_name: &str,
) -> Vec<ClientSegmentMetadata> {
    loop {
        let shard: GetShardMetadataRespShard = if let Some(shard) =
            metadata_cache.get_shard(shard_name)
        {
            shard
        } else {
            if let Err(e) = load_shards_cache(metadata_cache, connection_manager, shard_name).await
            {
                error!(
                    "Loading Shard {} Metadata info failed, error message :{}",
                    shard_name, e
                );
            }
            error!(
                "{}",
                JournalClientError::NotShardMetadata(shard_name.to_string()).to_string()
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        };

        return shard.segments;
    }
}
