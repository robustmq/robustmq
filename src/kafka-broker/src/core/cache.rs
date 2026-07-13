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

use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use kafka_protocol::error::ResponseError;
use metadata_struct::kafka::delegation_token::KafkaDelegationToken;
use metadata_struct::kafka::quota::{KafkaClientQuota, QUOTA_DEFAULT_NAME};
use metadata_struct::kafka::scram::KafkaScramCredential;

use crate::core::sasl::SaslSession;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::sync::oneshot;

use crate::core::assignor::TopicMeta;
use crate::core::consumer_group_meta::{self, ConsumerDescribedGroup, ConsumerGroupMeta};
use crate::core::consumer_heartbeat::{
    self, heartbeat_error, ConsumerHeartbeatParams, ConsumerHeartbeatResult,
};
use crate::core::group_admin::{self, DescribedGroupInfo, ListedGroupInfo};
use crate::core::group_meta::{GroupMeta, MemberMeta};
use crate::core::heartbeat;
use crate::core::join::{self, AddMemberOutcome, JoinCompletion};
use crate::core::leave::{self, LeaveOutcome};
use crate::core::sync::{self, sync_error, SyncOutcome, SyncResult};

// In-memory data the Kafka broker caches on the coordinator node (consumer-group
// state, keyed by group_id). Pure lock routing: each method takes the group's
// per-key lock and delegates to the phase logic in join/sync/leave/heartbeat.
/// One accepted idempotent batch: its base sequence and the base offset it was
/// written at, so a retry (which resends with the same base sequence) can be
/// answered with the original offset.
#[derive(Clone, Copy)]
struct BatchRecord {
    first_seq: i32,
    base_offset: i64,
}

/// Per (producer_id, shard) idempotence state: the current producer epoch, the
/// next expected base sequence, and a small window of recently-accepted batches
/// (Kafka keeps the last 5, matching the default max in-flight requests) so a
/// retry of any in-flight batch — not just the most recent — is deduped.
struct ProducerState {
    epoch: i16,
    next_seq: i32,
    recent: VecDeque<BatchRecord>,
}

/// How many recent batches to remember per producer for retry dedup — mirrors
/// Kafka's `max.in.flight.requests.per.connection` cap of 5 for idempotence.
const PRODUCER_DEDUP_WINDOW: usize = 5;

/// Outcome of an idempotent-batch sequence check.
pub enum SequenceCheck {
    /// In order — write it and record the new sequence range.
    Accept,
    /// Already written (retry within the window) — skip the write and return
    /// this recorded base offset.
    Duplicate(i64),
    /// A gap in, or a stale sequence outside the window — reject with
    /// OUT_OF_ORDER_SEQUENCE_NUMBER.
    OutOfOrder,
    /// The batch carries an older producer epoch than the one we've seen —
    /// reject with INVALID_PRODUCER_EPOCH (the producer has been fenced).
    Fenced,
}

#[derive(Default)]
pub struct KafkaCacheManager {
    // Classic-protocol groups.
    groups: DashMap<String, GroupMeta>,
    // KIP-848 consumer groups; a group id belongs to exactly one protocol.
    consumer_groups: DashMap<String, ConsumerGroupMeta>,
    // Client quotas, keyed by entity_key ("{entity_type}/{name|__default__}").
    quotas: DashMap<String, KafkaClientQuota>,
    // Delegation tokens (KIP-48), keyed by token_id. Metadata only — nothing
    // here verifies a token's `hmac`; see `KafkaDelegationToken`'s doc comment.
    delegation_tokens: DashMap<String, KafkaDelegationToken>,
    // SCRAM credentials, keyed by "{user}/{mechanism}".
    scram_credentials: DashMap<String, KafkaScramCredential>,
    // Per-connection SASL state, keyed by connection id.
    sasl_sessions: DashMap<u64, SaslSession>,
    // Idempotent-producer id allocator (InitProducerId). Broker-local monotonic
    // counter — fine for a single node; transactions/cluster-wide blocks are out
    // of scope.
    producer_id_counter: AtomicI64,
    // Per (producer_id, shard) sequence state for idempotent produce dedup.
    producer_sequences: DashMap<(i64, String), ProducerState>,
}

impl KafkaCacheManager {
    pub fn new() -> Self {
        KafkaCacheManager {
            groups: DashMap::with_capacity(8),
            consumer_groups: DashMap::with_capacity(8),
            quotas: DashMap::with_capacity(8),
            delegation_tokens: DashMap::with_capacity(8),
            scram_credentials: DashMap::with_capacity(8),
            sasl_sessions: DashMap::with_capacity(8),
            producer_id_counter: AtomicI64::new(1),
            producer_sequences: DashMap::with_capacity(8),
        }
    }

    /// Allocate a fresh idempotent-producer id (InitProducerId).
    pub fn next_producer_id(&self) -> i64 {
        self.producer_id_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Decide how to treat an incoming idempotent batch for this
    /// (producer_id, shard) at `epoch` with base sequence `base_seq`:
    /// - older epoch than seen  → `Fenced` (INVALID_PRODUCER_EPOCH)
    /// - newer epoch, or first batch, or the expected next sequence → `Accept`
    /// - a base sequence still in the recent window → `Duplicate(base_offset)`
    /// - anything else (gap ahead, or stale beyond the window) → `OutOfOrder`
    pub fn check_producer_sequence(
        &self,
        producer_id: i64,
        epoch: i16,
        shard: &str,
        base_seq: i32,
    ) -> SequenceCheck {
        match self
            .producer_sequences
            .get(&(producer_id, shard.to_string()))
        {
            None => SequenceCheck::Accept,
            Some(s) => {
                if epoch < s.epoch {
                    SequenceCheck::Fenced
                } else if epoch > s.epoch {
                    // A new producer incarnation resets the sequence space.
                    SequenceCheck::Accept
                } else if base_seq == s.next_seq {
                    SequenceCheck::Accept
                } else if let Some(b) = s.recent.iter().find(|b| b.first_seq == base_seq) {
                    // Retry of a batch still in the window: echo its offset.
                    SequenceCheck::Duplicate(b.base_offset)
                } else {
                    SequenceCheck::OutOfOrder
                }
            }
        }
    }

    /// Record an accepted idempotent batch, advancing the expected sequence and
    /// appending to the recent-batch window (a newer epoch resets the window).
    pub fn record_producer_sequence(
        &self,
        producer_id: i64,
        epoch: i16,
        shard: &str,
        first_seq: i32,
        last_seq: i32,
        base_offset: i64,
    ) {
        let mut state = self
            .producer_sequences
            .entry((producer_id, shard.to_string()))
            .or_insert_with(|| ProducerState {
                epoch,
                next_seq: 0,
                recent: VecDeque::with_capacity(PRODUCER_DEDUP_WINDOW),
            });
        if epoch > state.epoch {
            state.epoch = epoch;
            state.recent.clear();
        }
        state.recent.push_back(BatchRecord {
            first_seq,
            base_offset,
        });
        while state.recent.len() > PRODUCER_DEDUP_WINDOW {
            state.recent.pop_front();
        }
        state.next_seq = last_seq.wrapping_add(1);
    }

    pub fn set_sasl_session(&self, connection_id: u64, session: SaslSession) {
        self.sasl_sessions.insert(connection_id, session);
    }

    pub fn get_sasl_session(&self, connection_id: u64) -> Option<SaslSession> {
        self.sasl_sessions.get(&connection_id).map(|s| s.clone())
    }

    pub fn remove_sasl_session(&self, connection_id: u64) {
        self.sasl_sessions.remove(&connection_id);
    }

    pub fn is_sasl_authenticated(&self, connection_id: u64) -> bool {
        matches!(
            self.sasl_sessions.get(&connection_id).as_deref(),
            Some(SaslSession::Authenticated { .. })
        )
    }

    pub fn set_quota(&self, quota: KafkaClientQuota) {
        self.quotas.insert(quota.entity_key(), quota);
    }

    pub fn remove_quota(&self, entity_key: &str) {
        self.quotas.remove(entity_key);
    }

    // Effective quota for an entity: the specific entry, else the type default.
    pub fn get_quota(&self, entity_type: &str, name: &str) -> Option<KafkaClientQuota> {
        if let Some(q) = self.quotas.get(&format!("{}/{}", entity_type, name)) {
            return Some(q.clone());
        }
        self.quotas
            .get(&format!("{}/{}", entity_type, QUOTA_DEFAULT_NAME))
            .map(|q| q.clone())
    }

    pub fn set_scram_credential(&self, credential: KafkaScramCredential) {
        self.scram_credentials
            .insert(credential.entity_key(), credential);
    }

    pub fn remove_scram_credential(&self, entity_key: &str) {
        self.scram_credentials.remove(entity_key);
    }

    pub fn get_scram_credential(&self, user: &str, mechanism: i8) -> Option<KafkaScramCredential> {
        self.scram_credentials
            .get(&format!("{}/{}", user, mechanism))
            .map(|c| c.clone())
    }

    pub fn set_delegation_token(&self, token: KafkaDelegationToken) {
        self.delegation_tokens.insert(token.token_id.clone(), token);
    }

    pub fn remove_delegation_token(&self, token_id: &str) {
        self.delegation_tokens.remove(token_id);
    }

    // Not called yet — `kafka::delegation_token`'s Create/Renew/Expire/Describe
    // handlers all read from meta-service directly today, not this cache.
    // This is the lookup shape SASL delegation-token auth will need (a fast
    // local check against the presented `hmac` on every connection attempt,
    // where a meta-service round trip per connection would be too slow) —
    // kept here now because the cache is already being populated via notify
    // regardless, so the read side is what's actually missing.
    pub fn find_delegation_token_by_hmac(&self, hmac: &[u8]) -> Option<KafkaDelegationToken> {
        self.delegation_tokens
            .iter()
            .find(|entry| entry.value().hmac == hmac)
            .map(|entry| entry.value().clone())
    }

    pub fn has_consumer_group(&self, group_id: &str) -> bool {
        self.consumer_groups.contains_key(group_id)
    }

    pub fn add_member(&self, group_id: &str, member: MemberMeta) -> AddMemberOutcome {
        let mut group = self
            .groups
            .entry(group_id.to_string())
            .or_insert_with(|| GroupMeta::new(group_id.to_string()));
        join::register_member(&mut group, member)
    }

    pub fn finish_rebalance(&self, group_id: &str) -> JoinCompletion {
        self.groups
            .get_mut(group_id)
            .map(|mut group| join::complete_join(&mut group))
            .unwrap_or_default()
    }

    pub fn sync_member(
        &self,
        group_id: &str,
        member_id: &str,
        generation_id: i32,
        assignments: Vec<(String, bytes::Bytes)>,
    ) -> SyncOutcome {
        match self.groups.get_mut(group_id) {
            Some(mut group) => sync::sync(&mut group, member_id, generation_id, assignments),
            None => SyncOutcome::Immediate(sync_error(ResponseError::UnknownMemberId.code())),
        }
    }

    pub fn expire_sync(
        &self,
        group_id: &str,
        generation_id: i32,
    ) -> Vec<(oneshot::Sender<SyncResult>, SyncResult)> {
        self.groups
            .get_mut(group_id)
            .map(|mut group| sync::expire_sync(&mut group, generation_id))
            .unwrap_or_default()
    }

    pub fn heartbeat_member(&self, group_id: &str, member_id: &str, generation_id: i32) -> i16 {
        match self.groups.get_mut(group_id) {
            Some(mut group) => heartbeat::heartbeat(&mut group, member_id, generation_id),
            None => ResponseError::UnknownMemberId.code(),
        }
    }

    pub fn reap_expired_members(&self, now_ms: u128, consumer_session_timeout_ms: u64) {
        for mut group in self.groups.iter_mut() {
            heartbeat::remove_expired_members(&mut group, now_ms);
        }
        for mut group in self.consumer_groups.iter_mut() {
            consumer_heartbeat::remove_expired_members(
                &mut group,
                now_ms,
                consumer_session_timeout_ms,
            );
        }
    }

    pub fn remove_members(&self, group_id: &str, member_ids: &[String]) -> LeaveOutcome {
        match self.groups.get_mut(group_id) {
            Some(mut group) => leave::leave(&mut group, member_ids),
            None => LeaveOutcome {
                results: member_ids
                    .iter()
                    .map(|id| (id.clone(), ResponseError::UnknownMemberId.code()))
                    .collect(),
                sync_wakeups: Vec::new(),
            },
        }
    }

    pub fn describe_group(&self, group_id: &str) -> Option<DescribedGroupInfo> {
        self.groups
            .get(group_id)
            .map(|group| group_admin::describe(&group))
    }

    pub fn list_groups(&self) -> Vec<ListedGroupInfo> {
        let mut groups: Vec<ListedGroupInfo> = self
            .groups
            .iter()
            .map(|group| ListedGroupInfo {
                group_id: group.group_id.clone(),
                protocol_type: group.protocol_type.clone().unwrap_or_default(),
                state: group.state.name().to_string(),
                group_type: "classic".to_string(),
            })
            .collect();
        groups.extend(self.consumer_groups.iter().map(|group| ListedGroupInfo {
            group_id: group.group_id.clone(),
            protocol_type: "consumer".to_string(),
            state: group.state_name().to_string(),
            group_type: "consumer".to_string(),
        }));
        groups.sort_by(|a, b| a.group_id.cmp(&b.group_id));
        groups
    }

    pub fn consumer_heartbeat(
        &self,
        params: ConsumerHeartbeatParams,
        resolve_topic: &dyn Fn(&str) -> Option<TopicMeta>,
        now_ms: u128,
    ) -> ConsumerHeartbeatResult {
        if self.groups.contains_key(&params.group_id) {
            return heartbeat_error(
                ResponseError::GroupIdNotFound.code(),
                "group id is used by a classic-protocol group",
                &params.member_id,
            );
        }
        let group_id = params.group_id.clone();
        let mut group = self
            .consumer_groups
            .entry(group_id.clone())
            .or_insert_with(|| ConsumerGroupMeta::new(group_id));
        consumer_heartbeat::heartbeat(&mut group, params, resolve_topic, now_ms)
    }

    pub fn describe_consumer_group(
        &self,
        group_id: &str,
        tenant: &str,
    ) -> Option<ConsumerDescribedGroup> {
        self.consumer_groups
            .get(group_id)
            .map(|group| consumer_group_meta::describe(&group, tenant))
    }

    pub fn delete_group(&self, group_id: &str) -> i16 {
        match self.groups.entry(group_id.to_string()) {
            Entry::Occupied(entry) => {
                return if entry.get().members.is_empty() {
                    entry.remove();
                    0
                } else {
                    ResponseError::NonEmptyGroup.code()
                };
            }
            Entry::Vacant(_) => {}
        }
        match self.consumer_groups.entry(group_id.to_string()) {
            Entry::Occupied(entry) => {
                if entry.get().members.is_empty() {
                    entry.remove();
                    0
                } else {
                    ResponseError::NonEmptyGroup.code()
                }
            }
            Entry::Vacant(_) => ResponseError::GroupIdNotFound.code(),
        }
    }
}

#[cfg(test)]
mod producer_sequence_tests {
    use super::*;

    #[test]
    fn next_producer_id_is_monotonic() {
        let cache = KafkaCacheManager::new();
        let a = cache.next_producer_id();
        let b = cache.next_producer_id();
        assert!(b > a, "producer ids must strictly increase: {a} then {b}");
    }

    #[test]
    fn sequence_check_accepts_first_batch_and_next_in_order() {
        let cache = KafkaCacheManager::new();
        // No state yet: the first batch is always accepted.
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 0),
            SequenceCheck::Accept
        ));

        // Record a batch spanning sequences 0..=4 written at base offset 0.
        cache.record_producer_sequence(7, 0, "shard-0", 0, 4, 0);

        // The next in-order batch starts at last_seq + 1.
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 5),
            SequenceCheck::Accept
        ));
    }

    #[test]
    fn sequence_dedups_any_in_flight_retry_within_the_window() {
        let cache = KafkaCacheManager::new();
        // Three in-flight batches, each written at base offset == first_seq.
        cache.record_producer_sequence(7, 0, "shard-0", 0, 4, 0);
        cache.record_producer_sequence(7, 0, "shard-0", 5, 9, 5);
        cache.record_producer_sequence(7, 0, "shard-0", 10, 14, 10);

        // Next in-order batch is accepted.
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 15),
            SequenceCheck::Accept
        ));
        // A retry of ANY batch still in the window — not just the latest — is a
        // duplicate answered with that batch's own base offset.
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 0),
            SequenceCheck::Duplicate(0)
        ));
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 5),
            SequenceCheck::Duplicate(5)
        ));
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 10),
            SequenceCheck::Duplicate(10)
        ));
        // A gap ahead of the expected next sequence is out of order.
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 20),
            SequenceCheck::OutOfOrder
        ));
    }

    #[test]
    fn sequence_window_evicts_batches_beyond_the_last_five() {
        let cache = KafkaCacheManager::new();
        // Six batches: the first (0..=4) is pushed out of the 5-entry window.
        for i in 0..6 {
            let first = i * 5;
            cache.record_producer_sequence(7, 0, "shard-0", first, first + 4, first as i64);
        }
        // The evicted oldest batch can no longer be deduped → out of order.
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 0),
            SequenceCheck::OutOfOrder
        ));
        // A batch still in the window is still deduped.
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-0", 5),
            SequenceCheck::Duplicate(5)
        ));
    }

    #[test]
    fn epoch_fencing_rejects_old_and_resets_on_new() {
        let cache = KafkaCacheManager::new();
        cache.record_producer_sequence(7, 3, "shard-0", 0, 4, 0);

        // An older epoch is fenced.
        assert!(matches!(
            cache.check_producer_sequence(7, 2, "shard-0", 5),
            SequenceCheck::Fenced
        ));
        // A newer epoch is a fresh incarnation: its sequence space restarts.
        assert!(matches!(
            cache.check_producer_sequence(7, 4, "shard-0", 0),
            SequenceCheck::Accept
        ));
        cache.record_producer_sequence(7, 4, "shard-0", 0, 4, 100);
        // The old epoch stays fenced afterwards.
        assert!(matches!(
            cache.check_producer_sequence(7, 3, "shard-0", 5),
            SequenceCheck::Fenced
        ));
    }

    #[test]
    fn sequence_state_is_isolated_per_producer_and_shard() {
        let cache = KafkaCacheManager::new();
        cache.record_producer_sequence(7, 0, "shard-0", 0, 4, 0);
        // A different producer, and a different shard, both start fresh.
        assert!(matches!(
            cache.check_producer_sequence(8, 0, "shard-0", 0),
            SequenceCheck::Accept
        ));
        assert!(matches!(
            cache.check_producer_sequence(7, 0, "shard-1", 0),
            SequenceCheck::Accept
        ));
    }
}
