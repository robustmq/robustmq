---
name: isr-replication-drill
description: Run the ISR rolling-restart chaos drill for RobustMQ — 10 rounds of kill-one-node + write + restart + verify across a 3-replica shard. Use when the user wants to verify replica replication / ISR health, failover, or behaviour under churn — e.g. "演练 isr 副本同步", "验证三副本同步", "kill leader 演练", "验证 leader 切换/故障转移", "滚动重启/频繁切换 isr 是否正常", "test ISR replication", "副本同步是否正常".
---

# RobustMQ ISR Replication Drill

Verifies ISR (In-Sync Replica) replication, leader failover, and recovery under
repeated rolling restarts on a running 3-node cluster.

The single drill (`three_replica_chaos_rolling_kill`) covers all three flows the
user cares about:

1. **Write + observe** — acks=all writes in full ISR; all 3 replicas leo==hw==cumulative, lso==0.
2. **Kill (leader or follower), observe** — ISR shrinks to 2; leader switches if victim was leader;
   epoch advances; survivors hold all committed data; acks=all still commits on 2 survivors.
3. **Restart, observe follower role** — victim rejoins ISR as follower (not leader); all 3 replicas
   catch up to cumulative; all nodes agree on leader/epoch/ISR/leo/hw.

Runs 10 rounds; victim rotates n1→n2→n3→n1…, covering both follower-kill and
leader-kill scenarios in the same run.

## When to Use

- Verify a 3-replica shard genuinely places replicas across all 3 nodes
- Confirm followers replicate the leader's log (LEO converges, HW advances)
- Verify leader failover: kill leader → switch + no committed-data loss + epoch bump
- Verify ISR recovery: killed node rejoins as follower, offsets fully caught up
- Regression-check fixed defects (see "Defects This Guards" below)

## Key Semantics — what "ISR healthy" means

| Signal | Meaning | Healthy when |
|--------|---------|--------------|
| `segment.replicas.len()` | replicas actually placed | `== 3` always |
| `segment.isr` | in-sync replica set | all 3 node ids present |
| `replica.leo` | log end offset | every replica `== cumulative` |
| `replica.high_watermark` | committed/visible offset | every replica `== cumulative` |
| `replica.in_isr` / `available` | membership + reachable | all `true` |
| `segment.leader_epoch` | fencing term | monotonic; bumps on every leader switch |

> **Storage type.** ISR replication only works with `EngineRocksDB` or `EngineMemory`.
> `EngineSegment` returns `NotLeaderForPartition` on follower fetch — use `EngineRocksDB`.

> **HW lags by one fetch round.** A follower learns the leader's HW from the *next*
> fetch response. Always poll until convergence — never assert HW immediately after a write.

## Prerequisites

```bash
# Build the executable (NOT `cargo build -p broker-server` — that only builds the lib).
cargo build --bin broker-server
```

Cluster configs: `config/cluster/server-{1,2,3}.toml`
(broker_id 1/2/3; meta/grpc 1228/2228/3228; engine tcp 1779/2779/3779;
admin http 58080/58082/58083). Data dirs: `data/broker-{1,2,3}`.

Test: `tests/tests/engine/isr.rs::three_replica_chaos_rolling_kill` (`#[ignore]`).
Test package: **`robustmq-test`**.

## Drill Steps

### 0. Clean state (mandatory)

```bash
for c in 1 2 3; do p=$(pgrep -f "server-$c.toml"); [ -n "$p" ] && kill -INT "$p"; done
for c in 1 2 3; do while pgrep -f "server-$c.toml" >/dev/null; do sleep 1; done; done
rm -rf data/broker-1 data/broker-2 data/broker-3 data/logs /tmp/n1.log /tmp/n2.log /tmp/n3.log
```

> **NEVER `pkill -9`** — SIGKILL leaves an `ESTABLISHED` socket on port 1228 that
> `mac_agent` keeps alive, blocking the next `bind`. Always stop with `kill -INT`.

### 1. Start the 3-node cluster (staggered so node1 bootstraps first)

```bash
./target/debug/broker-server --conf config/cluster/server-1.toml > /tmp/n1.log 2>&1 &
sleep 8
./target/debug/broker-server --conf config/cluster/server-2.toml > /tmp/n2.log 2>&1 &
sleep 9
./target/debug/broker-server --conf config/cluster/server-3.toml > /tmp/n3.log 2>&1 &
sleep 13
```

Verify:
```bash
grep -oE "bootstrapping single-node cluster|Successfully joined cluster via peer|Meta Service cluster is ready|Failed to start GRPC" /tmp/n1.log /tmp/n2.log /tmp/n3.log | sort | uniq -c
curl -s http://localhost:58080/api/info | python3 -c "import sys,json;d=json.load(sys.stdin).get('data',{});print('nodes:',sorted(n.get('node_id') for n in d.get('broker_node_list',[])))"
```

Expect: node1 bootstrap, node2/3 joined, `nodes: [1, 2, 3]`.

### 2. Run the drill

```bash
cargo test -p robustmq-test three_replica_chaos_rolling_kill -- --ignored --nocapture
```

Each round:

1. `wait_full_and_caught_up` — precondition: all 3 replicas leo==hw==cumulative, lso==0, in ISR.
2. `write_acks_all(BATCH=30)` → cumulative += 30. Confirm all 3 replicas caught up.
3. `kill_node(victim)` (victim rotates n1→n2→n3→…).
4. Wait for victim to leave ISR; if was-leader, assert new leader elected, epoch advanced.
5. Assert survivors: available, leo==hw==cumulative, lso==0, replicas=={1,2,3}.
6. `write_acks_all(BATCH=30)` on 2 survivors (degraded ISR). Confirm both commit.
7. `restart_node(victim)`.
8. `wait_full_and_caught_up` — all 3 replicas back in ISR, fully caught up.
9. Assert **restarted node is follower** (not the new leader).
10. Cross-node check: all 3 nodes agree on leader/epoch/ISR/leo/hw.
11. Log blacklist scan.

After all 10 rounds: read back all `cumulative` records; final log scan.

Expected tail:
```
===== round 1: leader=n3, restarting n1 (was_leader=false), cumulative=30 =====
  n1 left ISR after 11.3s: leader=n3 epoch=0 isr=[2, 3]
  degraded write committed on 2 survivors at leo=hw=60
===== round 1 OK: leader=n3 epoch=0 isr=[1, 2, 3] all leo=hw=60 lso=0 =====
...
===== round 3: leader=n3, restarting n3 (was_leader=true), cumulative=150 =====
  n3 left ISR after 28.3s: leader=n1 epoch=1 isr=[1]
  degraded write committed on 2 survivors at leo=hw=180
===== round 3 OK: leader=n1 epoch=1 isr=[1, 2, 3] all leo=hw=180 lso=0 =====
...
CHAOS COMPLETE: 10 rounds, 600 committed records read back OK, 6 leader-epoch switches, all replicas consistent
```

Timings:
- Dead **follower** leaves ISR in ~6–11s (`replica_lag_time_max_ms`).
- Dead **leader** triggers switch in ~28–30s (heartbeat timeout).
- Follower rejoin after restart in ~4–10s (reconcile self-heal).

## Check Conditions (per round)

| Check | What it catches |
|-------|-----------------|
| `hw <= leo` on every `checked_detail` call | HW/offset bookkeeping corruption |
| `replicas == {1,2,3}` always | replica drop / unintended migration |
| ISR shrinks to `{survivors}` after kill | ISR not removing dead replica |
| ISR recovers to `{1,2,3}` after rejoin | failure to re-admit rejoining follower |
| `leader_epoch` monotonic, bumps on switch | fencing broken |
| All 3 nodes agree: leader, epoch, ISR, leo, hw | split-brain / stale metadata view |
| Survivors: leo==hw==cumulative after kill | committed data loss on failover |
| Restarted node is follower, not leader | spurious election on rejoin |
| acks=all commits in full (3) AND degraded (2) ISR | write stall under ISR churn |
| Final read-back == cumulative records | content lost despite correct counters |
| Log blacklist: no blacklisted lines | raft panics, watchdog kills, unexplained exits |

## Log Blacklist

Lines that must NEVER appear (each is an unambiguous defect):

| Pattern | Meaning |
|---------|---------|
| `"invalid state: expect"` | raft `purge_upto > snapshot` regression |
| `"last_log_id=None"` | raft log read back empty on restart |
| `"Clean the hole"` | raft storage hole |
| `"forcing exit"` | shutdown watchdog fired (ungraceful) |
| `"acks=all timed out"` | committed write never acked |
| `"NotEnoughReplicas"` | acks=all rejected by server |
| `"panicked at"` | any Rust panic |

Benign noise (NOT blacklisted): `Unreachable node`, fetcher retry, `reconcile: follower has no fetcher`, diverged-tail truncation on leader switch.

Quick scan:
```bash
for c in 1 2 3; do echo "=== n$c ==="; grep -E "invalid state: expect|last_log_id=None|Clean the hole|forcing exit|acks=all timed out|NotEnoughReplicas|panicked at" /tmp/n$c.log | head -5; done
```

## Pass / Fail Criteria

PASS only if all hold after all 10 rounds:

- All 10 rounds complete without assertion failure.
- Every replica: `available`, `in_isr`, `leo == hw == cumulative`, `lso == 0`.
- `segment.replicas` always `[1, 2, 3]`.
- `leader_epoch` monotonic across all rounds; advances on leader-kill rounds.
- All 3 nodes agree on leader/epoch/ISR/leo/hw after every round.
- Final read-back returns exactly `cumulative` (= `ROUNDS × 2 × BATCH` = 600) records.
- No blacklisted lines in any node log.

Common FAIL signatures:

| Symptom | Likely cause |
|---------|--------------|
| follower `leo` never reaches cumulative | replication pipe broken (fetcher thread / transport) |
| follower `hw` stuck at 0 while `leo` caught up | follower not applying `leader_hw` from fetch response |
| `isr` shrinks below 3 after rejoin | reconcile self-heal not firing / replica not re-admitted |
| `acks=all timed out` | write stall; ISR too small or fetcher stuck |
| `"NotEnoughReplicas"` | acks=all rejected; ISR collapsed below `min_in_sync_replicas` |
| `last_epoch` went backward | bug in epoch tracking / metadata sync |
| restarted node `assert_ne!(leader, victim)` fails | spurious election on rejoin |

## Defects This Drill Guards

Each defect below was found by this drill; a regression re-introduces it and the drill fails.

| Fix | Bug it prevents |
|-----|-----------------|
| `broker-core` heartbeat reports to **every** meta node | heartbeat table is per-meta-node; after a meta-leader change the new leader had never seen heartbeats → expired ALL nodes every `heartbeat_timeout_ms` → endless ISR churn |
| `isr_manager::compute_new_isr` shrinks by fetch **recency** (`last_fetch_ts`), not `leo >= leader_leo` | dead replica that was fully caught up kept `leo == leader_leo` vs a non-advancing leader → pinned in ISR forever → acks=all blocked |
| `dynamic_cache.rs` Create handler: `remove_leader_segment` when this node is NOT the new leader | demoted node kept segment in `leader_segments`, kept running ISR maintenance → stale proposals / divergent view |
| `dynamic_cache.rs` Create handler: only inits offsets when `get_offset_state().is_none()` | leader-switch "Create" notification reset `latest_offset→0` on a node that already held data → committed data lost |
| `WriteReqBody.timeout_ms` threaded into `batch_write` acks=all wait | acks=all reused 500ms `replica_fetch_max_wait_ms` as commit timeout → spurious timeout |
| `reconcile.rs` self-heal: start fetcher when follower has no `fetch_state` | restarted follower missed leader-switch notification → never resumed replication → never rejoined ISR |
| `isr/apply.rs::apply_as_follower` passes `leader_epoch_changed` flag | every ISR membership notification triggered `needs_truncation: true` → fetcher stopped for OffsetsForLeaderEpoch round-trip → raced against acks=all 30s window |
| `grpc-clients/src/utils.rs` 5s `PER_CALL_TIMEOUT` per attempt | `retry_call_inner` blocked 153s on a dead address during Raft snapshot install → acks=all in round 10 timed out |

## Cleanup

```bash
for c in 1 2 3; do p=$(pgrep -f "server-$c.toml"); [ -n "$p" ] && kill -INT "$p"; done
for c in 1 2 3; do while pgrep -f "server-$c.toml" >/dev/null; do sleep 1; done; done
```

## Troubleshooting

| Symptom | Cause / fix |
|---------|-------------|
| `Failed to start GRPC server on port 1228` but `lsof -iTCP:1228 -sTCP:LISTEN` empty | `pkill -9` left an orphan `ESTABLISHED` socket; `mac_agent` keeps it alive. Check `netstat -anp tcp \| grep '\.1228 '`. Kill the `mac_agent` pid (launchd respawns it) or reboot. Prevent with `kill -INT`. |
| Rebuilt code "didn't take" | `cargo build -p broker-server` builds only the lib. Use `cargo build --bin broker-server`. |
| `segment_detail` returns null | Segment not in cache yet — wait 10s and confirm `segment_seq` exists via the segment list. |
| `EngineSegment` replicas never sync | ISR replication not implemented for `EngineSegment`. Use `EngineRocksDB`. |

## Run Report

After each drill session emit a structured summary:

```
=== ISR REPLICATION DRILL RUN REPORT ===
Date:        <YYYY-MM-DD>
Code commit: <git rev-parse --short HEAD>

Drill — rolling-restart chaos (10 rounds)
  Result:    PASS / FAIL
  Rounds:    <N>/10 passed
  Records:   <N> committed records read back OK
  Epochs:    <N> leader-epoch switches
  Details:   <any notable anomalies or the failure round + panic line>

Log errors:
  node1: <N> ERRORs  (grep -c ' ERROR' /tmp/n1.log)
  node2: <N> ERRORs
  node3: <N> ERRORs

Overall:  PASS / FAIL
========================================
```

Gather inputs:
```bash
git rev-parse --short HEAD
for c in 1 2 3; do echo "node$c ERROR: $(grep -c ' ERROR' /tmp/n$c.log)"; done
```
