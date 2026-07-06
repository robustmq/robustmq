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

use crate::core::offset::ShardOffsetState;
use crate::core::offset_index::SegmentOffsetIndex;
use crate::filesegment::file::SegmentFile;
use crate::filesegment::SegmentIdentity;
use crate::isr::follower::SegmentReplicaState;
use broker_core::cache::NodeCacheManager;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use dashmap::DashMap;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Clone)]
pub struct StorageCacheManager {
    pub broker_cache: Arc<NodeCacheManager>,

    // --- Shard ---
    // shard_name -> EngineShard
    pub shards: DashMap<String, EngineShard>,

    // --- Segment ---
    // shard_name -> (segment_seq -> EngineSegment)
    pub segments: DashMap<String, DashMap<u32, EngineSegment>>,
    // segment_name -> SegmentIdentity  (only segments where this broker is leader)
    pub leader_segments: DashMap<String, SegmentIdentity>,

    // --- Segment Metadata (EngineSegment only) ---
    // shard_name -> (segment_seq -> EngineSegmentMetadata)
    pub segment_metadatas: DashMap<String, DashMap<u32, EngineSegmentMetadata>>,
    // shard_name -> SegmentOffsetIndex  (offset lookup index built from metadata)
    pub segment_offset_index: DashMap<String, SegmentOffsetIndex>,

    // --- Segment File (EngineSegment only) ---
    // segment_name -> SegmentFile
    pub segment_file_writer: DashMap<String, SegmentFile>,

    // --- Offset State ---
    // shard_name -> ShardOffsetState (earliest / latest / HW)
    pub shard_offset_state: DashMap<String, ShardOffsetState>,
    // shard_name -> HW watch channel  (wakes write waiters)
    pub hw_watchers: DashMap<String, watch::Sender<u64>>,

    // --- ISR Replica State ---
    // (shard_name, segment_seq) -> SegmentReplicaState
    pub segment_replica_states: DashMap<(String, u32), Arc<SegmentReplicaState>>,

    // --- Segment Scroll ---
    // shard_name -> next segment_seq being created (set while scroll is in progress)
    pub is_next_segment: DashMap<String, u32>,

    // --- Reconcile ---
    // (shard_name, segment_seq) -> last-trigger timestamp (seconds); used for rate-limiting
    pub reconcile_needed: DashMap<(String, u32), u64>,

    // --- Pending Deletes ---
    // Queues drained by delete.rs every 5 s.
    pub pending_delete_shards: Arc<Mutex<Vec<String>>>,
    pub pending_delete_segments: Arc<Mutex<Vec<SegmentIdentity>>>,
}

impl StorageCacheManager {
    pub fn new(broker_cache: Arc<NodeCacheManager>) -> Self {
        StorageCacheManager {
            broker_cache,
            shards: DashMap::with_capacity(8),
            segments: DashMap::with_capacity(8),
            leader_segments: DashMap::with_capacity(8),
            segment_metadatas: DashMap::with_capacity(8),
            segment_offset_index: DashMap::with_capacity(8),
            segment_file_writer: DashMap::with_capacity(2),
            shard_offset_state: DashMap::with_capacity(2),
            hw_watchers: DashMap::with_capacity(8),
            segment_replica_states: DashMap::with_capacity(8),
            is_next_segment: DashMap::with_capacity(2),
            reconcile_needed: DashMap::with_capacity(8),
            pending_delete_shards: Arc::new(Mutex::new(Vec::new())),
            pending_delete_segments: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // ── Shard ────────────────────────────────────────────────────────────────

    pub fn set_shard(&self, shard: EngineShard) {
        self.shards.insert(shard.shard_name.clone(), shard);
    }

    pub fn delete_shard(&self, shard_name: &str) {
        self.shards.remove(shard_name);
        self.segments.remove(shard_name);
        self.segment_metadatas.remove(shard_name);
        self.segment_offset_index.remove(shard_name);
        self.is_next_segment.remove(shard_name);
        self.shard_offset_state.remove(shard_name);
        self.hw_watchers.remove(shard_name);
        self.leader_segments
            .retain(|_, v| v.shard_name != shard_name);
        self.segment_file_writer
            .retain(|_, v| v.shard_name != shard_name);
        self.segment_replica_states
            .retain(|(shard, _), _| shard != shard_name);
        self.reconcile_needed
            .retain(|(shard, _), _| shard != shard_name);
    }

    // ── Segment ──────────────────────────────────────────────────────────────

    /// Insert or replace a segment. Automatically keeps `leader_segments` in sync.
    pub fn set_segment(&self, segment: &EngineSegment) {
        self.segments
            .entry(segment.shard_name.clone())
            .or_insert_with(|| DashMap::with_capacity(8))
            .insert(segment.segment_seq, segment.clone());

        let iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
        if broker_config().broker_id == segment.leader {
            self.leader_segments.insert(iden.name(), iden);
        } else {
            self.leader_segments.remove(&iden.name());
        }
    }

    pub fn delete_segment(&self, segment: &SegmentIdentity) {
        if let Some(list) = self.segments.get(&segment.shard_name) {
            list.remove(&segment.segment);
        }
        if let Some(list) = self.segment_metadatas.get(&segment.shard_name) {
            list.remove(&segment.segment);
        }
        self.leader_segments.remove(&segment.name());
        self.segment_file_writer.remove(&segment.name());
        if let Some(mut index) = self.segment_offset_index.get_mut(&segment.shard_name) {
            index.delete(segment.segment);
        }
        self.segment_replica_states
            .remove(&(segment.shard_name.clone(), segment.segment));
        self.reconcile_needed
            .remove(&(segment.shard_name.clone(), segment.segment));
    }

    pub fn get_segment(&self, segment: &SegmentIdentity) -> Option<EngineSegment> {
        self.segments
            .get(&segment.shard_name)?
            .get(&segment.segment)
            .map(|s| s.clone())
    }

    pub fn get_active_segment(&self, shard_name: &str) -> Option<EngineSegment> {
        let active_seq = self.shards.get(shard_name)?.active_segment_seq;
        self.get_segment(&SegmentIdentity::new(shard_name, active_seq))
    }

    pub fn get_segments_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegment> {
        self.segments
            .get(shard_name)
            .map(|list| list.iter().map(|r| r.clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_segment_leader_nodes(&self, shard_name: &str) -> Vec<u64> {
        let mut node_ids: Vec<u64> = self
            .get_segments_list_by_shard(shard_name)
            .iter()
            .map(|seg| seg.leader)
            .collect();
        node_ids.sort_unstable();
        node_ids.dedup();
        node_ids
    }

    // ── Segment Metadata ─────────────────────────────────────────────────────

    pub fn set_segment_meta(&self, segment: EngineSegmentMetadata) {
        self.segment_metadatas
            .entry(segment.shard_name.clone())
            .or_insert_with(|| DashMap::with_capacity(8))
            .insert(segment.segment_seq, segment.clone());

        self.segment_offset_index
            .entry(segment.shard_name.clone())
            .or_default()
            .add(
                segment.segment_seq,
                segment.start_offset,
                segment.start_timestamp,
                segment.end_timestamp,
            );
    }

    pub fn get_segment_meta(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Option<EngineSegmentMetadata> {
        self.segment_metadatas
            .get(&segment_iden.shard_name)?
            .get(&segment_iden.segment)
            .map(|s| s.clone())
    }

    pub fn update_start_meta(&self, segment_iden: &SegmentIdentity, offset: u64) {
        if let Some(list) = self.segment_metadatas.get(&segment_iden.shard_name) {
            if let Some(mut meta) = list.get_mut(&segment_iden.segment) {
                meta.start_offset = offset as i64;
                meta.start_timestamp = now_second() as i64;
            }
        }
    }

    // ── Offset Index ─────────────────────────────────────────────────────────

    pub fn get_offset_index(&self, shard_name: &str) -> Option<SegmentOffsetIndex> {
        self.segment_offset_index.get(shard_name).map(|e| e.clone())
    }

    pub fn sort_offset_index(&self, shard_name: &str) {
        if let Some(mut index) = self.segment_offset_index.get_mut(shard_name) {
            index.sort();
        }
    }

    // ── Offset State ─────────────────────────────────────────────────────────

    pub fn save_offset_state(&self, shard_name: String, offset_state: ShardOffsetState) {
        self.shard_offset_state.insert(shard_name, offset_state);
    }

    pub fn get_offset_state(&self, shard_name: &str) -> Option<ShardOffsetState> {
        self.shard_offset_state.get(shard_name).map(|s| s.clone())
    }

    pub fn update_latest_offset(&self, shard_name: &str, offset: u64) {
        if let Some(mut state) = self.shard_offset_state.get_mut(shard_name) {
            state.latest_offset = offset;
        }
    }

    pub fn update_earliest_offset(&self, shard_name: &str, offset: u64) {
        if let Some(mut state) = self.shard_offset_state.get_mut(shard_name) {
            state.earliest_offset = offset;
        }
    }

    pub fn update_high_watermark_offset(&self, shard_name: &str, offset: u64) -> bool {
        if let Some(mut state) = self.shard_offset_state.get_mut(shard_name) {
            if offset > state.high_watermark_offset {
                state.high_watermark_offset = offset;
                return true;
            }
        }
        false
    }

    pub fn hw_watcher(&self, shard_name: &str) -> watch::Sender<u64> {
        self.hw_watchers
            .entry(shard_name.to_string())
            .or_insert_with(|| watch::channel(0).0)
            .clone()
    }

    // ── ISR Replica State ────────────────────────────────────────────────────

    pub fn add_segment_replica(&self, shard: &str, segment_seq: u32) {
        self.segment_replica_states
            .entry((shard.to_string(), segment_seq))
            .or_insert_with(|| Arc::new(DashMap::new()));
    }

    pub fn get_segment_replica(
        &self,
        shard: &str,
        segment_seq: u32,
    ) -> Option<Arc<SegmentReplicaState>> {
        self.segment_replica_states
            .get(&(shard.to_string(), segment_seq))
            .map(|s| s.clone())
    }

    pub fn remove_segment_replica(&self, shard: &str, segment_seq: u32) {
        self.segment_replica_states
            .remove(&(shard.to_string(), segment_seq));
    }

    // ── Segment Scroll ───────────────────────────────────────────────────────

    pub fn add_next_segment(&self, shard: &str, segment: u32) {
        self.is_next_segment.insert(shard.to_string(), segment);
    }

    pub fn remove_next_segment(&self, shard: &str) {
        self.is_next_segment.remove(shard);
    }

    // ── Reconcile ────────────────────────────────────────────────────────────

    pub fn mark_reconcile_needed(&self, shard: &str, segment_seq: u32, min_interval_sec: u64) {
        let now = now_second();
        let mut entry = self
            .reconcile_needed
            .entry((shard.to_string(), segment_seq))
            .or_insert(0);
        if now.saturating_sub(*entry) >= min_interval_sec {
            *entry = now;
        }
    }

    pub fn take_reconcile_needed(&self) -> Vec<(String, u32)> {
        let keys: Vec<_> = self
            .reconcile_needed
            .iter()
            .map(|e| e.key().clone())
            .collect();
        self.reconcile_needed.clear();
        keys
    }

    // ── Pending Deletes ──────────────────────────────────────────────────────

    pub fn push_pending_delete_shard(&self, shard_name: String) {
        let mut q = self.pending_delete_shards.lock().unwrap();
        if !q.contains(&shard_name) {
            q.push(shard_name);
        }
    }

    pub fn push_pending_delete_segment(&self, seg_iden: SegmentIdentity) {
        let mut q = self.pending_delete_segments.lock().unwrap();
        if !q.contains(&seg_iden) {
            q.push(seg_iden);
        }
    }

    pub fn take_pending_deletes(&self) -> (Vec<String>, Vec<SegmentIdentity>) {
        let shards = std::mem::take(&mut *self.pending_delete_shards.lock().unwrap());
        let segments = std::mem::take(&mut *self.pending_delete_segments.lock().unwrap());
        (shards, segments)
    }
}
