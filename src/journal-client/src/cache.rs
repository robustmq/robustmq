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
use std::time::Duration;

use dashmap::DashMap;
use log::error;
use metadata_struct::journal::segment::segment_name;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal_server::journal_engine::{
    ClientSegmentMetadata, GetClusterMetadataNode, GetShardMetadataReqShard,
    GetShardMetadataRespShard,
};
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;

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

    pub fn get_shard(&self, namespace: &str, shard: &str) -> Option<GetShardMetadataRespShard> {
        if let Some(shard) = self.shards.get(&shard_name_iden(namespace, shard)) {
            return Some(shard.clone());
        }
        None
    }
    pub fn add_shard(&self, shard: GetShardMetadataRespShard) {
        self.shards
            .insert(shard_name_iden(&shard.namespace, &shard.shard), shard);
    }

    pub fn get_active_segment(&self, namespace: &str, shard: &str) -> Option<u32> {
        let key = shard_name_iden(namespace, shard);
        if let Some(shard) = self.shards.get(&key) {
            return Some(shard.active_segment as u32);
        }
        None
    }

    pub fn get_segment_leader(&self, namespace: &str, shard: &str) -> Option<u32> {
        let key = shard_name_iden(namespace, shard);
        if let Some(shard) = self.shards.get(&key) {
            return Some(shard.active_segment as u32);
        }
        None
    }

    pub fn get_leader_by_segment(&self, namespace: &str, shard: &str, segment: u32) -> Option<u64> {
        let key = shard_name_iden(namespace, shard);
        if let Some(shard) = self.shards.get(&key) {
            for meta in shard.segments.iter() {
                if meta.segment_no == segment {
                    return Some(meta.leader);
                }
            }
        }
        None
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
    namespace: &str,
    shard_name: &str,
) -> Result<(), JournalClientError> {
    let shards = vec![GetShardMetadataReqShard {
        namespace: namespace.to_owned(),
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

pub fn start_update_cache_thread(
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
        if let Err(e) = load_shards_cache(
            &metadata_cache,
            &connection_manager,
            &shard.namespace,
            &shard.shard,
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
    sleep(Duration::from_secs(10)).await;
}

pub async fn get_active_segment(
    metadata_cache: &Arc<MetadataCache>,
    connection_manager: &Arc<ConnectionManager>,
    namespace: &str,
    shard_name: &str,
) -> u32 {
    loop {
        let active_segment = if let Some(segment) =
            metadata_cache.get_active_segment(namespace, shard_name)
        {
            segment
        } else {
            if let Err(e) =
                load_shards_cache(metadata_cache, connection_manager, namespace, shard_name).await
            {
                error!(
                    "Loading Shard {} Metadata info failed, error message :{}",
                    shard_name_iden(namespace, shard_name,),
                    e
                );
            }
            error!(
                "{}",
                JournalClientError::NotActiveSegment(shard_name_iden(namespace, shard_name))
                    .to_string()
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
    namespace: &str,
    shard_name: &str,
) -> u64 {
    let active_segment =
        get_active_segment(metadata_cache, connection_manager, namespace, shard_name).await;

    loop {
        let leader = if let Some(leader) = metadata_cache.get_segment_leader(namespace, shard_name)
        {
            leader
        } else {
            if let Err(e) =
                load_shards_cache(metadata_cache, connection_manager, namespace, shard_name).await
            {
                error!(
                    "Loading Shard {} Metadata info failed, error message :{}",
                    shard_name_iden(namespace, shard_name),
                    e
                );
            }
            error!(
                "{}",
                JournalClientError::NotLeader(segment_name(namespace, shard_name, active_segment))
                    .to_string()
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
    namespace: &str,
    shard_name: &str,
) -> Vec<ClientSegmentMetadata> {
    loop {
        let shard: GetShardMetadataRespShard = if let Some(shard) =
            metadata_cache.get_shard(namespace, shard_name)
        {
            shard
        } else {
            if let Err(e) =
                load_shards_cache(metadata_cache, connection_manager, namespace, shard_name).await
            {
                error!(
                    "Loading Shard {} Metadata info failed, error message :{}",
                    shard_name_iden(namespace, shard_name),
                    e
                );
            }
            error!(
                "{}",
                JournalClientError::NotShardMetadata(shard_name_iden(namespace, shard_name,))
                    .to_string()
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        };

        return shard.segments;
    }
}
