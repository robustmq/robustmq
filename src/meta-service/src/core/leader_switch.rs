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

use crate::{
    core::{
        cache::MetaCacheManager,
        error::MetaServiceError,
        group_leader::generate_group_leader,
        notify::{send_notify_by_set_segment, send_notify_by_set_share_group},
        segment::sync_save_segment_info,
    },
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
};
use bytes::Bytes;
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use tracing::{error, info};

pub async fn trigger_leader_switch(
    meta_cache: Arc<MetaCacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_call_manager: Arc<NodeCallManager>,
    remove_id: u64,
) {
    tokio::spawn(async move {
        let result: Result<(), MetaServiceError> = async {
            group_leader_switch(
                &meta_cache,
                &raft_manager,
                &mqtt_call_manager,
                &rocksdb_engine_handler,
                remove_id,
            )
            .await?;
            segment_leader_switch(&meta_cache, &raft_manager, &mqtt_call_manager, remove_id)
                .await?;
            Ok(())
        }
        .await;
        if let Err(e) = result {
            error!("leader switch failed for removed node {}: {}", remove_id, e);
        }
    });
}

pub async fn group_leader_switch(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    remove_id: u64,
) -> Result<(), MetaServiceError> {
    let affected: Vec<_> = meta_cache
        .group_leader
        .iter()
        .filter(|g| g.leader_broker == remove_id)
        .map(|g| g.clone())
        .collect();

    let mut switched = 0u32;
    for mut group_leader in affected {
        let new_leader_broker =
            generate_group_leader(meta_cache, rocksdb_engine_handler, &group_leader.tenant).await?;
        group_leader.leader_broker = new_leader_broker;

        let data = StorageData::new(
            StorageDataType::MqttSetGroupLeader,
            Bytes::copy_from_slice(&group_leader.encode()?),
        );
        raft_manager
            .write_data(&group_leader.group_name, data)
            .await?;
        send_notify_by_set_share_group(call_manager, group_leader).await?;
        switched += 1;
    }
    info!(
        "group_leader_switch completed, node {} removed, {} group leaders switched",
        remove_id, switched
    );
    Ok(())
}

pub async fn segment_leader_switch(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    remove_id: u64,
) -> Result<(), MetaServiceError> {
    let affected: Vec<_> = meta_cache
        .segment_list
        .iter()
        .flat_map(|shard| {
            shard
                .iter()
                .filter(|seg| seg.leader == remove_id)
                .map(|seg| seg.clone())
                .collect::<Vec<_>>()
        })
        .collect();

    let mut switched = 0u32;
    for segment in affected {
        let Some(new_leader) = segment
            .replicas
            .iter()
            .find(|rep| rep.node_id != remove_id)
            .map(|rep| rep.node_id)
        else {
            error!(
                "segment {}/{} has no available replica to elect as leader, skipping",
                segment.shard_name, segment.segment_seq
            );
            continue;
        };

        let mut new_segment = segment.clone();
        new_segment.leader_epoch_incr();
        new_segment.leader = new_leader;

        sync_save_segment_info(raft_manager, &new_segment).await?;
        send_notify_by_set_segment(call_manager, new_segment).await?;
        switched += 1;
    }
    info!(
        "segment_leader_switch completed, node {} removed, {} segment leaders switched",
        remove_id, switched
    );
    Ok(())
}
