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

// T3 — isr_maintain_logic
// T4 — recovery_election_logic

#[cfg(test)]
mod tests {
    use meta_service::core::isr_recovery::{elect_recovery_leader, ReplicaStateReport};
    use storage_engine::isr::follower::{update_follower_progress, SegmentReplicaState};
    use storage_engine::isr::isr_maintain::compute_new_isr;

    // T3: compute_new_isr correctly shrinks and expands the ISR based on follower lag.
    #[test]
    fn isr_maintain_shrink_and_expand() {
        let state = SegmentReplicaState::new();
        let replicas = vec![1u64, 2];
        let leader_leo: u64 = 10;
        let lag_max_ms: u64 = 5_000; // 5 s
        let now_sec: u64 = 1_000_000;

        // Follower 2 last caught up 100 s ago — well beyond the 5 s window
        let stale_ts = now_sec - 100;
        update_follower_progress(&state, 2, 1, 0, leader_leo, stale_ts);

        // ISR should shrink: follower 2 is lagging and the window has expired
        let shrunk = compute_new_isr(
            &state, &replicas, &replicas, 1, leader_leo, lag_max_ms, now_sec,
        );
        assert_eq!(
            shrunk,
            Some(vec![1]),
            "stale follower should be evicted from ISR"
        );

        // Follower 2 catches up: update LEO to leader_leo, timestamp = now
        update_follower_progress(&state, 2, 1, leader_leo, leader_leo, now_sec);

        // ISR should expand back to [1, 2]
        let expanded = compute_new_isr(&state, &[1], &replicas, 1, leader_leo, lag_max_ms, now_sec);
        assert_eq!(
            expanded,
            Some(vec![1, 2]),
            "caught-up follower should rejoin ISR"
        );
    }

    // T3b: leader is never evicted even when no follower progress entry exists.
    #[test]
    fn isr_maintain_leader_always_in_isr() {
        let state = SegmentReplicaState::new();
        let replicas = vec![1u64];
        let now_sec: u64 = 1_000_000;

        let result = compute_new_isr(&state, &replicas, &replicas, 1, 5, 5_000, now_sec);
        // ISR is unchanged → returns None (no update needed)
        assert_eq!(result, None, "single-leader ISR should be stable");
    }

    // T3c: no change returns None rather than a new vec
    #[test]
    fn isr_maintain_returns_none_when_unchanged() {
        let state = SegmentReplicaState::new();
        let replicas = vec![1u64, 2];
        let leader_leo: u64 = 5;
        let now_sec: u64 = 1_000_000;

        // Follower 2 is fresh
        update_follower_progress(&state, 2, 1, leader_leo, leader_leo, now_sec);

        let result = compute_new_isr(&state, &replicas, &replicas, 1, leader_leo, 5_000, now_sec);
        assert_eq!(result, None, "unchanged ISR should return None");
    }

    // T4: elect_recovery_leader picks the replica with the highest LEO that is available.
    #[test]
    fn recovery_election_picks_highest_available_leo() {
        let reports = vec![
            ReplicaStateReport {
                replica_id: 1,
                segment_leo: 100,
                latest_leader_epoch: 3,
                available: true,
            },
            ReplicaStateReport {
                replica_id: 2,
                segment_leo: 80,
                latest_leader_epoch: 3,
                available: true,
            },
            ReplicaStateReport {
                replica_id: 3,
                segment_leo: 50,
                latest_leader_epoch: 3,
                available: false,
            },
        ];
        assert_eq!(
            elect_recovery_leader(&reports),
            Some(1),
            "replica with highest LEO should win"
        );
    }

    // T4b: highest LEO but unavailable → pick next best
    #[test]
    fn recovery_election_skips_unavailable() {
        let reports = vec![
            ReplicaStateReport {
                replica_id: 1,
                segment_leo: 200,
                latest_leader_epoch: 3,
                available: false, // highest LEO but down
            },
            ReplicaStateReport {
                replica_id: 2,
                segment_leo: 150,
                latest_leader_epoch: 3,
                available: true,
            },
        ];
        assert_eq!(
            elect_recovery_leader(&reports),
            Some(2),
            "unavailable replica should be skipped"
        );
    }

    // T4c: all unavailable → None
    #[test]
    fn recovery_election_returns_none_when_all_down() {
        let reports = vec![
            ReplicaStateReport {
                replica_id: 1,
                segment_leo: 100,
                latest_leader_epoch: 3,
                available: false,
            },
            ReplicaStateReport {
                replica_id: 2,
                segment_leo: 80,
                latest_leader_epoch: 3,
                available: false,
            },
        ];
        assert_eq!(elect_recovery_leader(&reports), None);
    }

    // T4d: tie in LEO → higher leader_epoch wins
    #[test]
    fn recovery_election_tie_broken_by_leader_epoch() {
        let reports = vec![
            ReplicaStateReport {
                replica_id: 1,
                segment_leo: 100,
                latest_leader_epoch: 2,
                available: true,
            },
            ReplicaStateReport {
                replica_id: 2,
                segment_leo: 100,
                latest_leader_epoch: 3,
                available: true,
            },
        ];
        assert_eq!(
            elect_recovery_leader(&reports),
            Some(2),
            "higher leader_epoch should break the tie"
        );
    }
}
