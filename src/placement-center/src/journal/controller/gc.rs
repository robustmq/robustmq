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

use grpc_clients::journal::inner::call::{
    journal_inner_delete_segment_file, journal_inner_delete_shard_file,
    journal_inner_get_segment_delete_status, journal_inner_get_shard_delete_status,
};
use grpc_clients::pool::ClientPool;
use log::{error, warn};
use metadata_struct::journal::segment::SegmentStatus;
use metadata_struct::journal::shard::JournalShardStatus;
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileRequest, DeleteShardFileRequest, GetSegmentDeleteStatusRequest,
    GetShardDeleteStatusRequest,
};

use crate::core::cache::PlacementCacheManager;
use crate::journal::cache::JournalCacheManager;
use crate::journal::services::segmet::{
    sync_delete_segment_info, sync_delete_segment_metadata_info, update_segment_status,
};
use crate::journal::services::shard::{
    sync_delete_shard_info, update_shard_status, update_start_segment_by_shard,
};
use crate::route::apply::RaftMachineApply;

pub async fn gc_shard_thread(
    raft_machine_apply: Arc<RaftMachineApply>,
    engine_cache: Arc<JournalCacheManager>,
    cluster_cache: Arc<PlacementCacheManager>,
    client_pool: Arc<ClientPool>,
) {
    for shard in engine_cache.wait_delete_shard_list.iter() {
        if shard.status != JournalShardStatus::PrepareDelete {
            warn!(
                "shard {} in wait_delete_shard_list is in the wrong state, current state is {:?}",
                shard.name(),
                shard.status
            );
            continue;
        }

        // to deleteing
        if let Err(e) = update_shard_status(
            &raft_machine_apply,
            &engine_cache,
            &shard.clone(),
            JournalShardStatus::Deleteing,
        )
        .await
        {
            error!(
                "Failed to convert Shard to deleteing state with error message: {}",
                e
            );
            continue;
        }

        let node_addrs = cluster_cache.get_broker_node_addr_by_cluster(&shard.cluster_name);

        // call all jen delete shard
        for node_addr in node_addrs.iter() {
            let addrs = vec![node_addr.to_string()];
            let request = DeleteShardFileRequest {
                cluster_name: shard.cluster_name.clone(),
                namespace: shard.namespace.clone(),
                shard_name: shard.shard_name.clone(),
            };
            if let Err(e) =
                journal_inner_delete_shard_file(client_pool.clone(), addrs, request).await
            {
                error!(
                    "Calling node {} to delete the Shard file failed with error message :{}",
                    node_addr, e
                );
            }
        }

        // get delete shard status
        let mut flag = true;
        for node_addr in node_addrs {
            let addrs = vec![node_addr.to_string()];
            let request = GetShardDeleteStatusRequest {
                cluster_name: shard.cluster_name.clone(),
                namespace: shard.namespace.clone(),
                shard_name: shard.shard_name.clone(),
            };
            match journal_inner_get_shard_delete_status(client_pool.clone(), addrs, request).await {
                Ok(reply) => {
                    if !reply.status {
                        flag = false;
                    }
                }
                Err(e) => {
                    error!("Calling node {} to get progress information on removing Shard failed, error message :{}", node_addr,e);
                }
            }
        }

        // delete shard/segment by storage/cache
        if !flag {
            let key =
                engine_cache.shard_key(&shard.cluster_name, &shard.namespace, &shard.shard_name);

            // delete segment
            if let Some(segment_list) = engine_cache.segment_list.get(&key) {
                for segment in segment_list.iter() {
                    if let Err(e) =
                        sync_delete_segment_info(&raft_machine_apply, &segment.clone()).await
                    {
                        error!(
                            "Failed to delete data from Segment {} with error message {}",
                            segment.name(),
                            e
                        );
                    };
                }
            }

            // delete segment meta
            if let Some(list) = engine_cache.segment_meta_list.get(&key) {
                for segment in list.iter() {
                    if let Err(e) =
                        sync_delete_segment_metadata_info(&raft_machine_apply, &segment.clone())
                            .await
                    {
                        error!(
                            "Failed to delete data from Segment {} with error message {}",
                            segment.name(),
                            e
                        );
                    };
                }
            }

            // delete shard
            if let Err(e) = sync_delete_shard_info(&raft_machine_apply, &shard).await {
                error!(
                    "Failed to delete Shard {} data with error message :{}",
                    shard.name(),
                    e
                );
            };

            engine_cache.remove_wait_delete_shard(&shard);
        }
    }
}

pub async fn gc_segment_thread(
    raft_machine_apply: Arc<RaftMachineApply>,
    engine_cache: Arc<JournalCacheManager>,
    cluster_cache: Arc<PlacementCacheManager>,
    client_pool: Arc<ClientPool>,
) {
    for segment in engine_cache.wait_delete_segment_list.iter() {
        if segment.status != SegmentStatus::PrepareDelete {
            warn!(
                "segment {} in wait_delete_segment_list is in the wrong state, current state is {:?}",
                segment.name(),
                segment.status
            );
            continue;
        }

        let mut shard = if let Some(shard) = engine_cache.get_shard(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
        ) {
            shard
        } else {
            engine_cache.remove_wait_delete_segment(&segment);
            continue;
        };

        // to deleteing
        if let Err(e) = update_segment_status(
            &engine_cache,
            &raft_machine_apply,
            &segment.clone(),
            SegmentStatus::Deleteing,
        )
        .await
        {
            error!(
                "Failed to convert Segment to deleteing state with error message: {}",
                e
            );
        }

        let node_ids: Vec<u64> = segment.replicas.iter().map(|rep| rep.node_id).collect();

        // call all jen delete segment
        for node_id in node_ids.iter() {
            if let Some(node) = cluster_cache.get_broker_node(&segment.cluster_name, *node_id) {
                let addrs = vec![node.node_inner_addr.clone()];
                let request = DeleteSegmentFileRequest {
                    cluster_name: segment.cluster_name.clone(),
                    namespace: segment.namespace.clone(),
                    shard_name: segment.shard_name.clone(),
                    segment: segment.segment_seq,
                };
                if let Err(e) =
                    journal_inner_delete_segment_file(client_pool.clone(), addrs, request).await
                {
                    error!(
                        "Calling node {} to delete the Segment file failed with error message :{}",
                        node.node_inner_addr, e
                    );
                }
            }
        }

        // get delete segment file status
        let mut flag = true;
        for node_id in node_ids.iter() {
            if let Some(node) = cluster_cache.get_broker_node(&segment.cluster_name, *node_id) {
                let addrs = vec![node.node_inner_addr.clone()];
                let request = GetSegmentDeleteStatusRequest {
                    cluster_name: segment.cluster_name.clone(),
                    namespace: segment.namespace.clone(),
                    shard_name: segment.shard_name.clone(),
                    segment: segment.segment_seq,
                };
                match journal_inner_get_segment_delete_status(client_pool.clone(), addrs, request)
                    .await
                {
                    Ok(reply) => {
                        if !reply.status {
                            flag = false;
                        }
                    }
                    Err(e) => {
                        error!("Calling node {} to get progress information on removing Segment failed, error message :{}", node.node_inner_addr,e);
                    }
                }
            }
        }

        // update info
        if !flag {
            // delete segment
            if let Err(e) = sync_delete_segment_info(&raft_machine_apply, &segment).await {
                error!(
                    "Failed to delete Segment {} data with error message :{}",
                    segment.name(),
                    e
                );
            };

            // delete segment meta
            if let Some(meta) = engine_cache.get_segment_meta(
                &segment.cluster_name,
                &segment.namespace,
                &segment.shard_name,
                segment.segment_seq,
            ) {
                if let Err(e) = sync_delete_segment_metadata_info(&raft_machine_apply, &meta).await
                {
                    error!(
                        "Failed to delete Segment metadata {} data with error message :{}",
                        segment.name(),
                        e
                    );
                };
            }

            // update start segment by shard
            if let Err(e) = update_start_segment_by_shard(
                &raft_machine_apply,
                &engine_cache,
                &mut shard,
                segment.segment_seq,
            )
            .await
            {
                error!(
                    "Updating the Shard {} start segment information failed with error message {}",
                    shard.name(),
                    e
                );
            }

            engine_cache.remove_wait_delete_segment(&segment);
        }
    }
}
