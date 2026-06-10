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

use crate::client::AdminHttpClient;
use crate::path::{api_path, STORAGE_ENGINE_SEGMENT_REPLICA_STATE_PATH};
use crate::state::HttpState;
use axum::{extract::State, Json};
use common_base::http_response::success_response;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use storage_engine::filesegment::SegmentIdentity;
use tracing::warn;

// ── list ──────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentListReq {
    pub shard_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentListResp {
    pub segment_list: Vec<SegmentListRespRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentListRespRaw {
    pub segment: EngineSegment,
    pub segment_meta: Option<EngineSegmentMetadata>,
}

pub async fn segment_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SegmentListReq>,
) -> String {
    let segment_list = state
        .engine_context
        .cache_manager
        .get_segments_list_by_shard(&params.shard_name);

    let mut results: Vec<_> = Vec::new();
    for segment in segment_list {
        let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
        let meta = state
            .engine_context
            .cache_manager
            .get_segment_meta(&segment_iden);
        results.push(SegmentListRespRaw {
            segment: segment.clone(),
            segment_meta: meta,
        });
    }
    success_response(SegmentListResp {
        segment_list: results,
    })
}

// ── detail ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentDetailReq {
    pub shard_name: String,
    pub segment_seq: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FollowerProgressResp {
    pub node_id: u64,
    pub leo: u64,
    pub last_caught_up_ts: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentReplicaStateResp {
    pub node_id: u64,
    pub is_leader: bool,
    pub in_isr: bool,
    pub role: String,
    pub leader_epoch: u32,
    pub segment_epoch: u32,
    pub leo: u64,
    pub high_watermark: u64,
    pub log_start_offset: u64,
    pub follower_progress: Vec<FollowerProgressResp>,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentDetailResp {
    pub segment: EngineSegment,
    pub segment_meta: Option<EngineSegmentMetadata>,
    pub replicas: Vec<SegmentReplicaStateResp>,
}

fn local_replica_state_from_cache(
    state: &Arc<HttpState>,
    shard_name: &str,
    segment_seq: u32,
) -> SegmentReplicaStateResp {
    let cm = &state.engine_context.cache_manager;
    let broker_id = state.broker_cache.get_cluster_config().broker_id;
    let offset_state = cm.get_offset_state(shard_name);
    let replica_state = cm.get_segment_replica(shard_name, segment_seq);

    let (leo, hw, log_start) = match &offset_state {
        Some(s) => (s.latest_offset, s.high_watermark_offset, s.earliest_offset),
        None => (0, 0, 0),
    };

    let segment_iden = SegmentIdentity::new(shard_name, segment_seq);
    let seg = cm.get_segment(&segment_iden);
    let role_str = match &seg {
        Some(s) if s.leader == broker_id => "Leader".to_string(),
        Some(_) => "Follower".to_string(),
        None => "Unknown".to_string(),
    };
    let segment_epoch = seg.as_ref().map(|s| s.segment_epoch).unwrap_or(0);

    let leader_epoch = seg.as_ref().map(|s| s.leader_epoch).unwrap_or(0);
    let follower_progress = match &replica_state {
        Some(rs) => rs
            .iter()
            .map(|e| FollowerProgressResp {
                node_id: *e.key(),
                leo: e.value().leo,
                last_caught_up_ts: e.value().last_fetch_ts,
            })
            .collect(),
        None => vec![],
    };

    SegmentReplicaStateResp {
        node_id: broker_id,
        is_leader: false,
        in_isr: false,
        role: role_str,
        leader_epoch,
        segment_epoch,
        leo,
        high_watermark: hw,
        log_start_offset: log_start,
        follower_progress,
        available: replica_state.is_some() || offset_state.is_some(),
        error: None,
    }
}

pub async fn segment_detail(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SegmentDetailReq>,
) -> String {
    let segment_iden = SegmentIdentity::new(&params.shard_name, params.segment_seq);

    let segment = match state
        .engine_context
        .cache_manager
        .get_segment(&segment_iden)
    {
        Some(s) => s,
        None => {
            return success_response::<Option<()>>(None);
        }
    };
    let segment_meta = state
        .engine_context
        .cache_manager
        .get_segment_meta(&segment_iden);

    let local_broker_id = state.broker_cache.get_cluster_config().broker_id;
    let req = SegmentDetailReq {
        shard_name: params.shard_name.clone(),
        segment_seq: params.segment_seq,
    };
    let mut replicas: Vec<SegmentReplicaStateResp> = Vec::with_capacity(segment.replicas.len());

    for replica in &segment.replicas {
        let node_id = replica.node_id;

        let mut resp = if node_id == local_broker_id {
            local_replica_state_from_cache(&state, &params.shard_name, params.segment_seq)
        } else {
            let node = state.broker_cache.node_lists.get(&node_id);
            let http_addr = match &node {
                Some(n) => n.http_addr.clone(),
                None => {
                    replicas.push(SegmentReplicaStateResp {
                        node_id,
                        available: false,
                        error: Some(format!("node {} not found in broker cache", node_id)),
                        is_leader: segment.leader == node_id,
                        in_isr: segment.isr.contains(&node_id),
                        role: String::new(),
                        leader_epoch: 0,
                        segment_epoch: 0,
                        leo: 0,
                        high_watermark: 0,
                        log_start_offset: 0,
                        follower_progress: vec![],
                    });
                    continue;
                }
            };
            drop(node);

            match AdminHttpClient::new(&http_addr)
                .post::<SegmentDetailReq, SegmentReplicaStateResp>(
                    &api_path(STORAGE_ENGINE_SEGMENT_REPLICA_STATE_PATH),
                    &req,
                )
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    warn!(
                        "segment_detail: failed to fetch replica state from node {} ({}): {}",
                        node_id, http_addr, e
                    );
                    replicas.push(SegmentReplicaStateResp {
                        node_id,
                        available: false,
                        error: Some(e.to_string()),
                        is_leader: segment.leader == node_id,
                        in_isr: segment.isr.contains(&node_id),
                        role: String::new(),
                        leader_epoch: 0,
                        segment_epoch: 0,
                        leo: 0,
                        high_watermark: 0,
                        log_start_offset: 0,
                        follower_progress: vec![],
                    });
                    continue;
                }
            }
        };

        resp.is_leader = segment.leader == node_id;
        resp.in_isr = segment.isr.contains(&node_id);
        replicas.push(resp);
    }

    success_response(SegmentDetailResp {
        segment,
        segment_meta,
        replicas,
    })
}

pub async fn segment_replica_state(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SegmentDetailReq>,
) -> String {
    success_response(local_replica_state_from_cache(
        &state,
        &params.shard_name,
        params.segment_seq,
    ))
}
