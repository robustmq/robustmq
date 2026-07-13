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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use common_base::tools::now_millis;
use kafka_protocol::error::ResponseError;

use crate::core::assignor::TopicMeta;
use crate::core::cache::KafkaCacheManager;
use crate::core::consumer_group_meta::ConsumerDescribedGroup;
use crate::core::consumer_heartbeat::{ConsumerHeartbeatParams, ConsumerHeartbeatResult};
use crate::core::group_admin::{DescribedGroupInfo, ListedGroupInfo};
use crate::core::group_meta::MemberMeta;
use crate::core::heartbeat::spawn_session_reaper;
use crate::core::join::{generate_member_id, join_error, JoinGroupParams, JoinResult, JoinTimer};
use crate::core::sync::{sync_error, SyncGroupParams, SyncOutcome, SyncResult};

const DEFAULT_INITIAL_REBALANCE_DELAY_MS: u64 = 3000;
const DEFAULT_REBALANCE_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_SESSION_CHECK_INTERVAL_MS: u64 = 1000;
const DEFAULT_CONSUMER_SESSION_TIMEOUT_MS: u64 = 45_000;
const DEFAULT_CONSUMER_HEARTBEAT_INTERVAL_MS: i32 = 5000;

pub struct GroupCoordinatorConfig {
    pub initial_rebalance_delay_ms: u64,
    pub rebalance_timeout_ms: u64,
    pub session_check_interval_ms: u64,
    // KIP-848: session timeout is a server-side group config, not a request field.
    pub consumer_session_timeout_ms: u64,
    pub consumer_heartbeat_interval_ms: i32,
}

impl Default for GroupCoordinatorConfig {
    fn default() -> Self {
        GroupCoordinatorConfig {
            initial_rebalance_delay_ms: DEFAULT_INITIAL_REBALANCE_DELAY_MS,
            rebalance_timeout_ms: DEFAULT_REBALANCE_TIMEOUT_MS,
            session_check_interval_ms: DEFAULT_SESSION_CHECK_INTERVAL_MS,
            consumer_session_timeout_ms: DEFAULT_CONSUMER_SESSION_TIMEOUT_MS,
            consumer_heartbeat_interval_ms: DEFAULT_CONSUMER_HEARTBEAT_INTERVAL_MS,
        }
    }
}

pub struct GroupCoordinator {
    cache: Arc<KafkaCacheManager>,
    config: GroupCoordinatorConfig,
    reaper_started: AtomicBool,
}

impl GroupCoordinator {
    pub fn new(cache: Arc<KafkaCacheManager>) -> Self {
        Self::new_with_config(cache, GroupCoordinatorConfig::default())
    }

    pub fn new_with_config(cache: Arc<KafkaCacheManager>, config: GroupCoordinatorConfig) -> Self {
        GroupCoordinator {
            cache,
            config,
            reaper_started: AtomicBool::new(false),
        }
    }

    // The coordinator is constructed outside the tokio runtime during broker startup,
    // so the reaper is spawned lazily from the first group request instead.
    fn ensure_reaper_started(&self) {
        if !self.reaper_started.swap(true, Ordering::SeqCst) {
            spawn_session_reaper(
                self.cache.clone(),
                self.config.session_check_interval_ms,
                self.config.consumer_session_timeout_ms,
            );
        }
    }

    pub async fn join_group(&self, params: JoinGroupParams) -> JoinResult {
        self.ensure_reaper_started();
        if self.cache.has_consumer_group(&params.group_id) {
            return join_error(
                ResponseError::InconsistentGroupProtocol.code(),
                params.member_id.clone(),
            );
        }
        if params.member_id.is_empty() && params.require_member_id {
            return join_error(
                ResponseError::MemberIdRequired.code(),
                generate_member_id(&params.client_id),
            );
        }
        let member_id = if params.member_id.is_empty() {
            generate_member_id(&params.client_id)
        } else {
            params.member_id.clone()
        };

        let member = MemberMeta {
            member_id: member_id.clone(),
            group_instance_id: params.group_instance_id.clone(),
            client_id: params.client_id.clone(),
            client_host: params.client_host.clone(),
            session_timeout_ms: params.session_timeout_ms,
            rebalance_timeout_ms: params.rebalance_timeout_ms,
            protocol_type: params.protocol_type.clone(),
            protocols: params.protocols.clone(),
            assignment: Bytes::new(),
            join_waiter: None,
            sync_waiter: None,
            last_heartbeat_ms: now_millis(),
        };
        let outcome = self.cache.add_member(&params.group_id, member);

        for (tx, result) in outcome.sync_wakeups {
            let _ = tx.send(result);
        }
        if outcome.complete_now {
            finish_join_phase(
                self.cache.clone(),
                &params.group_id,
                self.config.rebalance_timeout_ms,
            );
        } else if let Some(timer) = outcome.timer {
            let delay = match timer {
                JoinTimer::InitialDelay => self.config.initial_rebalance_delay_ms,
                JoinTimer::RebalanceTimeout(0) => self.config.rebalance_timeout_ms,
                JoinTimer::RebalanceTimeout(ms) => ms,
            };
            self.spawn_join_timer(params.group_id.clone(), delay);
        }

        outcome
            .join_rx
            .await
            .unwrap_or_else(|_| join_error(ResponseError::RebalanceInProgress.code(), member_id))
    }

    pub async fn sync_group(&self, params: SyncGroupParams) -> SyncResult {
        match self.cache.sync_member(
            &params.group_id,
            &params.member_id,
            params.generation_id,
            params.assignments,
        ) {
            SyncOutcome::Immediate(result) => result,
            SyncOutcome::Completed { own, wakeups } => {
                for (tx, result) in wakeups {
                    let _ = tx.send(result);
                }
                own
            }
            SyncOutcome::Park(rx) => rx
                .await
                .unwrap_or_else(|_| sync_error(ResponseError::RebalanceInProgress.code())),
        }
    }

    pub fn heartbeat(&self, group_id: &str, member_id: &str, generation_id: i32) -> i16 {
        self.cache
            .heartbeat_member(group_id, member_id, generation_id)
    }

    pub fn leave_group(&self, group_id: &str, member_ids: &[String]) -> Vec<(String, i16)> {
        let outcome = self.cache.remove_members(group_id, member_ids);
        for (tx, result) in outcome.sync_wakeups {
            let _ = tx.send(result);
        }
        outcome.results
    }

    pub fn describe_group(&self, group_id: &str) -> Option<DescribedGroupInfo> {
        self.cache.describe_group(group_id)
    }

    pub fn list_groups(&self) -> Vec<ListedGroupInfo> {
        self.cache.list_groups()
    }

    pub fn delete_group(&self, group_id: &str) -> i16 {
        self.cache.delete_group(group_id)
    }

    pub fn consumer_heartbeat(
        &self,
        params: ConsumerHeartbeatParams,
        resolve_topic: &dyn Fn(&str) -> Option<TopicMeta>,
    ) -> ConsumerHeartbeatResult {
        self.ensure_reaper_started();
        self.cache
            .consumer_heartbeat(params, resolve_topic, now_millis())
    }

    pub fn describe_consumer_group(
        &self,
        group_id: &str,
        tenant: &str,
    ) -> Option<ConsumerDescribedGroup> {
        self.cache.describe_consumer_group(group_id, tenant)
    }

    pub fn consumer_heartbeat_interval_ms(&self) -> i32 {
        self.config.consumer_heartbeat_interval_ms
    }

    fn spawn_join_timer(&self, group_id: String, delay_ms: u64) {
        let cache = self.cache.clone();
        let sync_timeout = self.config.rebalance_timeout_ms;
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            finish_join_phase(cache, &group_id, sync_timeout);
        });
    }
}

fn finish_join_phase(cache: Arc<KafkaCacheManager>, group_id: &str, sync_timeout_ms: u64) {
    let completion = cache.finish_rebalance(group_id);
    for (tx, result) in completion.deliveries {
        let _ = tx.send(result);
    }
    if let Some(generation) = completion.completing_generation {
        spawn_sync_timer(cache, group_id.to_string(), generation, sync_timeout_ms);
    }
}

fn spawn_sync_timer(
    cache: Arc<KafkaCacheManager>,
    group_id: String,
    generation: i32,
    timeout_ms: u64,
) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
        for (tx, result) in cache.expire_sync(&group_id, generation) {
            let _ = tx.send(result);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_coordinator() -> GroupCoordinator {
        GroupCoordinator::new_with_config(
            Arc::new(KafkaCacheManager::new()),
            GroupCoordinatorConfig {
                initial_rebalance_delay_ms: 50,
                rebalance_timeout_ms: 300,
                session_check_interval_ms: 50,
                consumer_session_timeout_ms: 45_000,
                consumer_heartbeat_interval_ms: 5000,
            },
        )
    }

    fn params(group: &str, member_id: &str, protocols: Vec<&str>) -> JoinGroupParams {
        JoinGroupParams {
            group_id: group.to_string(),
            member_id: member_id.to_string(),
            group_instance_id: None,
            client_id: "c".to_string(),
            client_host: "/127.0.0.1".to_string(),
            session_timeout_ms: 30_000,
            rebalance_timeout_ms: 60_000,
            protocol_type: "consumer".to_string(),
            protocols: protocols
                .into_iter()
                .map(|n| (n.to_string(), Bytes::new()))
                .collect(),
            require_member_id: false,
        }
    }

    #[tokio::test]
    async fn empty_member_id_on_v4_requires_member_id() {
        let coord = fast_coordinator();
        let mut p = params("g", "", vec!["range"]);
        p.require_member_id = true;
        let r = coord.join_group(p).await;
        assert_eq!(r.error_code, ResponseError::MemberIdRequired.code());
        assert!(!r.member_id.is_empty());
    }

    #[tokio::test]
    async fn two_members_share_one_generation_and_leader() {
        let coord = Arc::new(fast_coordinator());
        let c1 = coord.clone();
        let c2 = coord.clone();
        let h1 = tokio::spawn(async move { c1.join_group(params("g", "m1", vec!["range"])).await });
        let h2 = tokio::spawn(async move { c2.join_group(params("g", "m2", vec!["range"])).await });
        let (r1, r2) = (h1.await.unwrap(), h2.await.unwrap());

        assert_eq!(r1.error_code, 0);
        assert_eq!(r2.error_code, 0);
        assert_eq!(r1.generation_id, 1);
        assert_eq!(r2.generation_id, 1);
        assert_eq!(r1.leader_id, r2.leader_id);
        assert_eq!(r1.protocol_name.as_deref(), Some("range"));

        let leader = &r1.leader_id;
        let (leader_res, follower_res) = if &r1.member_id == leader {
            (&r1, &r2)
        } else {
            (&r2, &r1)
        };
        assert_eq!(leader_res.members.len(), 2);
        assert!(follower_res.members.is_empty());
    }

    #[tokio::test]
    async fn incompatible_protocols_fail_the_group() {
        let coord = Arc::new(fast_coordinator());
        let c1 = coord.clone();
        let c2 = coord.clone();
        let h1 = tokio::spawn(async move { c1.join_group(params("g", "m1", vec!["range"])).await });
        let h2 =
            tokio::spawn(async move { c2.join_group(params("g", "m2", vec!["roundrobin"])).await });
        let (r1, r2) = (h1.await.unwrap(), h2.await.unwrap());

        assert_eq!(
            r1.error_code,
            ResponseError::InconsistentGroupProtocol.code()
        );
        assert_eq!(
            r2.error_code,
            ResponseError::InconsistentGroupProtocol.code()
        );
    }

    #[tokio::test]
    async fn common_protocol_selected_across_members() {
        let coord = Arc::new(fast_coordinator());
        let c1 = coord.clone();
        let c2 = coord.clone();
        let h1 = tokio::spawn(async move {
            c1.join_group(params("g", "m1", vec!["sticky", "range"]))
                .await
        });
        let h2 = tokio::spawn(async move {
            c2.join_group(params("g", "m2", vec!["range", "roundrobin"]))
                .await
        });
        let (r1, _r2) = (h1.await.unwrap(), h2.await.unwrap());
        assert_eq!(r1.error_code, 0);
        assert_eq!(r1.protocol_name.as_deref(), Some("range"));
    }

    fn sync_params(
        group: &str,
        member: &str,
        gen: i32,
        assignments: Vec<(&str, &[u8])>,
    ) -> SyncGroupParams {
        SyncGroupParams {
            group_id: group.to_string(),
            member_id: member.to_string(),
            generation_id: gen,
            assignments: assignments
                .into_iter()
                .map(|(m, a)| (m.to_string(), Bytes::copy_from_slice(a)))
                .collect(),
        }
    }

    // Join two members, returning (coordinator, leader_id, follower_id, generation).
    async fn joined_group(group: &str) -> (Arc<GroupCoordinator>, String, String, i32) {
        let coord = Arc::new(fast_coordinator());
        let c1 = coord.clone();
        let c2 = coord.clone();
        let g1 = group.to_string();
        let g2 = group.to_string();
        let h1 = tokio::spawn(async move { c1.join_group(params(&g1, "m1", vec!["range"])).await });
        let h2 = tokio::spawn(async move { c2.join_group(params(&g2, "m2", vec!["range"])).await });
        let (r1, r2) = (h1.await.unwrap(), h2.await.unwrap());
        let leader = r1.leader_id.clone();
        let follower = if r1.member_id == leader {
            r2.member_id
        } else {
            r1.member_id
        };
        (coord, leader, follower, r1.generation_id)
    }

    #[tokio::test]
    async fn leader_sync_assigns_and_wakes_followers() {
        let (coord, leader, follower, gen) = joined_group("g").await;

        let c = coord.clone();
        let f = follower.clone();
        let follower_sync =
            tokio::spawn(async move { c.sync_group(sync_params("g", &f, gen, vec![])).await });

        // Give the follower a moment to park, then the leader syncs the assignment.
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let leader_res = coord
            .sync_group(sync_params(
                "g",
                &leader,
                gen,
                vec![(leader.as_str(), b"L"), (follower.as_str(), b"F")],
            ))
            .await;

        assert_eq!(leader_res.error_code, 0);
        assert_eq!(leader_res.assignment.as_ref(), b"L");

        let follower_res = follower_sync.await.unwrap();
        assert_eq!(follower_res.error_code, 0);
        assert_eq!(follower_res.assignment.as_ref(), b"F");
    }

    #[tokio::test]
    async fn resync_on_stable_group_returns_stored_assignment() {
        let (coord, leader, follower, gen) = joined_group("g").await;
        coord
            .sync_group(sync_params(
                "g",
                &leader,
                gen,
                vec![(follower.as_str(), b"F")],
            ))
            .await;
        // Follower syncs after the group is already Stable → gets its stored assignment.
        let res = coord
            .sync_group(sync_params("g", &follower, gen, vec![]))
            .await;
        assert_eq!(res.error_code, 0);
        assert_eq!(res.assignment.as_ref(), b"F");
    }

    #[tokio::test]
    async fn sync_with_wrong_generation_is_fenced() {
        let (coord, leader, _follower, gen) = joined_group("g").await;
        let res = coord
            .sync_group(sync_params("g", &leader, gen + 5, vec![]))
            .await;
        assert_eq!(res.error_code, ResponseError::IllegalGeneration.code());
    }

    #[tokio::test]
    async fn follower_sync_times_out_when_leader_never_syncs() {
        let (coord, _leader, follower, gen) = joined_group("g").await;
        // Leader never syncs; the follower should be woken by the sync timeout.
        let res = coord
            .sync_group(sync_params("g", &follower, gen, vec![]))
            .await;
        assert_eq!(res.error_code, ResponseError::RebalanceInProgress.code());
    }

    async fn stable_single(
        coord: &GroupCoordinator,
        group: &str,
        session_timeout_ms: i32,
    ) -> (String, i32) {
        let jr = coord
            .join_group(JoinGroupParams {
                group_id: group.to_string(),
                member_id: String::new(),
                group_instance_id: None,
                client_id: "c".to_string(),
                client_host: "/127.0.0.1".to_string(),
                session_timeout_ms,
                rebalance_timeout_ms: 60_000,
                protocol_type: "consumer".to_string(),
                protocols: vec![("range".to_string(), Bytes::new())],
                require_member_id: false,
            })
            .await;
        let member = jr.member_id;
        let gen = jr.generation_id;
        coord
            .sync_group(sync_params(
                group,
                &member,
                gen,
                vec![(member.as_str(), b"A")],
            ))
            .await;
        (member, gen)
    }

    #[tokio::test]
    async fn heartbeat_ok_when_stable() {
        let coord = fast_coordinator();
        let (member, gen) = stable_single(&coord, "g", 30_000).await;
        assert_eq!(coord.heartbeat("g", &member, gen), 0);
    }

    #[tokio::test]
    async fn heartbeat_fencing() {
        let coord = fast_coordinator();
        let (member, gen) = stable_single(&coord, "g", 30_000).await;
        assert_eq!(
            coord.heartbeat("g", &member, gen + 9),
            ResponseError::IllegalGeneration.code()
        );
        assert_eq!(
            coord.heartbeat("g", "no-such-member", gen),
            ResponseError::UnknownMemberId.code()
        );
    }

    #[tokio::test]
    async fn heartbeat_signals_rebalance_before_sync() {
        let (coord, leader, _follower, gen) = joined_group("g").await;
        assert_eq!(
            coord.heartbeat("g", &leader, gen),
            ResponseError::RebalanceInProgress.code()
        );
    }

    #[tokio::test]
    async fn leave_triggers_rebalance_for_survivors() {
        let (coord, leader, follower, gen) = joined_group("g").await;
        coord
            .sync_group(sync_params(
                "g",
                &leader,
                gen,
                vec![(leader.as_str(), b"L"), (follower.as_str(), b"F")],
            ))
            .await;

        let results = coord.leave_group("g", std::slice::from_ref(&leader));
        assert_eq!(results, vec![(leader.clone(), 0)]);
        assert_eq!(
            coord.heartbeat("g", &follower, gen),
            ResponseError::RebalanceInProgress.code()
        );
    }

    #[tokio::test]
    async fn leave_unknown_member_is_rejected() {
        let coord = fast_coordinator();
        let (_member, _gen) = stable_single(&coord, "g", 30_000).await;
        let results = coord.leave_group("g", &["ghost".to_string()]);
        assert_eq!(results[0].1, ResponseError::UnknownMemberId.code());
        let results = coord.leave_group("no-such-group", &["m".to_string()]);
        assert_eq!(results[0].1, ResponseError::UnknownMemberId.code());
    }

    #[tokio::test]
    async fn leader_leave_wakes_parked_sync_followers() {
        let (coord, leader, follower, gen) = joined_group("g").await;

        let c = coord.clone();
        let f = follower.clone();
        let parked =
            tokio::spawn(async move { c.sync_group(sync_params("g", &f, gen, vec![])).await });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        coord.leave_group("g", std::slice::from_ref(&leader));
        let res = parked.await.unwrap();
        assert_eq!(res.error_code, ResponseError::RebalanceInProgress.code());
    }

    #[tokio::test]
    async fn new_join_during_completing_wakes_parked_sync() {
        let (coord, _leader, follower, gen) = joined_group("g").await;

        let c = coord.clone();
        let f = follower.clone();
        let parked =
            tokio::spawn(async move { c.sync_group(sync_params("g", &f, gen, vec![])).await });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // A third member joins mid-sync: the parked follower must be released.
        let c3 = coord.clone();
        tokio::spawn(async move { c3.join_group(params("g", "m3", vec!["range"])).await });

        let res = tokio::time::timeout(std::time::Duration::from_millis(100), parked)
            .await
            .expect("parked sync must be woken by the new join")
            .unwrap();
        assert_eq!(res.error_code, ResponseError::RebalanceInProgress.code());
    }

    #[tokio::test]
    async fn new_member_join_moves_stable_group_into_rebalance() {
        let coord = Arc::new(fast_coordinator());
        let (member, gen) = stable_single(&coord, "g", 30_000).await;

        let c2 = coord.clone();
        tokio::spawn(async move { c2.join_group(params("g", "m2", vec!["range"])).await });
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        assert_eq!(
            coord.heartbeat("g", &member, gen),
            ResponseError::RebalanceInProgress.code()
        );
    }

    #[tokio::test]
    async fn describe_reflects_group_state_and_members() {
        let coord = fast_coordinator();
        let (member, _gen) = stable_single(&coord, "g", 30_000).await;

        let info = coord.describe_group("g").unwrap();
        assert_eq!(info.state, "Stable");
        assert_eq!(info.protocol_type, "consumer");
        assert_eq!(info.protocol_data, "range");
        assert_eq!(info.members.len(), 1);
        assert_eq!(info.members[0].member_id, member);
        assert_eq!(info.members[0].member_assignment.as_ref(), b"A");

        assert!(coord.describe_group("no-such-group").is_none());
    }

    #[tokio::test]
    async fn list_groups_returns_state() {
        let coord = fast_coordinator();
        let (_member, _gen) = stable_single(&coord, "g1", 30_000).await;
        let listed = coord.list_groups();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].group_id, "g1");
        assert_eq!(listed[0].state, "Stable");
    }

    #[tokio::test]
    async fn delete_group_semantics() {
        let coord = fast_coordinator();
        let (member, _gen) = stable_single(&coord, "g", 30_000).await;

        assert_eq!(coord.delete_group("g"), ResponseError::NonEmptyGroup.code());
        coord.leave_group("g", &[member]);
        assert_eq!(coord.delete_group("g"), 0);
        assert_eq!(
            coord.delete_group("g"),
            ResponseError::GroupIdNotFound.code()
        );
    }

    fn params_with_timeout(
        group: &str,
        member_id: &str,
        rebalance_timeout_ms: i32,
    ) -> JoinGroupParams {
        let mut p = params(group, member_id, vec!["range"]);
        p.rebalance_timeout_ms = rebalance_timeout_ms;
        p
    }

    #[tokio::test]
    async fn non_rejoining_member_is_dropped_on_rebalance_completion() {
        // Build a stable pair with a short rebalance timeout.
        let coord = Arc::new(fast_coordinator());
        let c1 = coord.clone();
        let c2 = coord.clone();
        let h1 =
            tokio::spawn(async move { c1.join_group(params_with_timeout("g", "m1", 200)).await });
        let h2 =
            tokio::spawn(async move { c2.join_group(params_with_timeout("g", "m2", 200)).await });
        let (r1, r2) = (h1.await.unwrap(), h2.await.unwrap());
        let leader = r1.leader_id.clone();
        let follower = if r1.member_id == leader {
            r2.member_id.clone()
        } else {
            r1.member_id.clone()
        };
        coord
            .sync_group(sync_params("g", &leader, r1.generation_id, vec![]))
            .await;

        // Leader leaves; the follower never rejoins (a zombie). A new member joins:
        // the rebalance must complete without the zombie once its timeout elapses.
        coord.leave_group("g", std::slice::from_ref(&leader));
        let r3 = coord.join_group(params_with_timeout("g", "m3", 200)).await;

        assert_eq!(r3.error_code, 0);
        assert_eq!(r3.leader_id, r3.member_id);
        assert_eq!(r3.members.len(), 1);
        assert_eq!(
            coord.heartbeat("g", &follower, r3.generation_id),
            ResponseError::UnknownMemberId.code()
        );
    }

    #[tokio::test]
    async fn rebalance_completes_early_when_all_members_rejoin() {
        let (coord, leader, follower, gen) = joined_group("g").await;
        coord
            .sync_group(sync_params(
                "g",
                &leader,
                gen,
                vec![(leader.as_str(), b"L"), (follower.as_str(), b"F")],
            ))
            .await;

        // m3 joins (rebalance timeout is 60s); both existing members rejoin promptly.
        // Completion must come from "all rejoined", far before any timer.
        let c3 = coord.clone();
        let h3 = tokio::spawn(async move { c3.join_group(params("g", "m3", vec!["range"])).await });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let cl = coord.clone();
        let l = leader.clone();
        let hl = tokio::spawn(async move { cl.join_group(params("g", &l, vec!["range"])).await });
        let cf = coord.clone();
        let f = follower.clone();
        let hf = tokio::spawn(async move { cf.join_group(params("g", &f, vec!["range"])).await });

        let all = async move { (h3.await.unwrap(), hl.await.unwrap(), hf.await.unwrap()) };
        let (r3, rl, rf) = tokio::time::timeout(std::time::Duration::from_secs(2), all)
            .await
            .expect("join phase must complete early when everyone rejoined");

        assert_eq!(r3.error_code, 0);
        assert_eq!(rl.error_code, 0);
        assert_eq!(rf.error_code, 0);
        assert_eq!(r3.generation_id, gen + 1);
        assert_eq!(rl.leader_id, leader);
        let leader_res = [&r3, &rl, &rf]
            .into_iter()
            .find(|r| r.member_id == r.leader_id)
            .unwrap();
        assert_eq!(leader_res.members.len(), 3);
    }

    #[tokio::test]
    async fn classic_join_rejected_when_consumer_group_owns_the_id() {
        let coord = fast_coordinator();
        // Create a KIP-848 group with the same id.
        let resolve = |_: &str| None;
        coord.consumer_heartbeat(
            crate::core::consumer_heartbeat::ConsumerHeartbeatParams {
                group_id: "g".to_string(),
                member_id: "cm1".to_string(),
                member_epoch: 0,
                instance_id: None,
                rack_id: None,
                client_id: "c".to_string(),
                rebalance_timeout_ms: 60_000,
                subscribed_topics: Some(vec!["t".to_string()]),
                server_assignor: None,
                owned: None,
            },
            &resolve,
        );

        let r = coord.join_group(params("g", "m1", vec!["range"])).await;
        assert_eq!(
            r.error_code,
            ResponseError::InconsistentGroupProtocol.code()
        );
    }

    #[tokio::test]
    async fn delete_group_covers_consumer_groups() {
        let coord = fast_coordinator();
        let resolve = |_: &str| None;
        let hb = |epoch: i32| crate::core::consumer_heartbeat::ConsumerHeartbeatParams {
            group_id: "g".to_string(),
            member_id: "cm1".to_string(),
            member_epoch: epoch,
            instance_id: None,
            rack_id: None,
            client_id: "c".to_string(),
            rebalance_timeout_ms: 60_000,
            subscribed_topics: Some(vec!["t".to_string()]),
            server_assignor: None,
            owned: None,
        };
        coord.consumer_heartbeat(hb(0), &resolve);

        assert_eq!(coord.delete_group("g"), ResponseError::NonEmptyGroup.code());
        coord.consumer_heartbeat(hb(-1), &resolve);
        assert_eq!(coord.delete_group("g"), 0);
        assert_eq!(
            coord.delete_group("g"),
            ResponseError::GroupIdNotFound.code()
        );
    }

    #[tokio::test]
    async fn session_timeout_evicts_member() {
        let coord = fast_coordinator();
        let (member, gen) = stable_single(&coord, "g", 100).await;
        assert_eq!(coord.heartbeat("g", &member, gen), 0);
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        assert_eq!(
            coord.heartbeat("g", &member, gen),
            ResponseError::UnknownMemberId.code()
        );
    }
}
