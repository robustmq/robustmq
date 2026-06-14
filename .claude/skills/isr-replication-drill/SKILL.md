---
name: isr-replication-drill
description: Run 3-node ISR drills for RobustMQ — replica-sync (LEO/HW convergence), leader-failover (kill leader → switch + no committed-data loss + rejoin), and rolling-restart chaos (repeatedly restart one node, frequent leader switches, verify ISR stays correct). Use when the user wants to verify replica replication / ISR health, failover, or behaviour under churn — e.g. "演练 isr 副本同步", "验证三副本同步", "kill leader 演练", "验证 leader 切换/故障转移", "滚动重启/频繁切换 isr 是否正常", "test ISR replication", "副本同步是否正常".
---

# RobustMQ ISR Replication Drill

A reproducible drill to verify ISR (In-Sync Replica) replication works end-to-end
on a running RobustMQ cluster: create a 3-replica topic, write data continuously
to the leader, and confirm every follower catches up (LEO converges, HW advances,
ISR stays full) by polling the segment-detail admin API.

## When to Use

- Verify a 3-replica shard genuinely places 3 replicas across 3 nodes
- Confirm followers replicate the leader's log (follower LEO → leader LEO)
- Confirm the high watermark (HW) advances on **both** leader and followers
- Confirm the ISR set stays full (no replica falls out) under continuous writes
- Verify leader **failover**: killing the leader switches it to a surviving replica
  with NO committed-data loss, and the killed node rejoins the ISR after restart
  (Drill B)
- Verify ISR correctness under a **rolling restart** with frequent leader switches
  (Drill C)
- Regression-check the fixes: follower-HW (apply `leader_hw` from the fetch
  response), segment-detail cross-node fan-out, leader-switch offset reset
  (committed-data loss), acks=all producer commit timeout, reconcile self-heal of a
  restarted follower's fetcher, heartbeat reported to every meta node (no false
  expiry after a meta-leader change), recency-based ISR shrink (drop a dead
  caught-up follower), and `leader_segments` cleanup on demotion

## Key Semantics — what "ISR healthy" means

| Signal | Meaning | Healthy when |
|--------|---------|--------------|
| `segment.replicas.len()` | replicas actually placed | `== replica_num` (3) |
| `segment.isr` | in-sync replica set | contains all 3 node ids |
| `replica.leo` | log end offset of that replica | every follower `== leader.leo` |
| `replica.high_watermark` | committed/visible offset | every replica `== leader.leo` after convergence |
| `replica.in_isr` / `available` | membership + reachable | all `true` |
| `leader.high_watermark` | leader committed offset | `== leader.leo` once ISR all ack |

The three convergence conditions together prove health:
1. **LEO converges** → replication pipe works (followers pulled every record).
2. **HW converges** → commit semantics work (leader advances HW once ISR acks; the
   follower then learns it from the next fetch response and advances its local HW).
3. **ISR stays = 3** → no replica fell behind / got removed.

> **CRITICAL — storage type.** ISR leader/follower fetch replication applies ONLY to
> `EngineRocksDB` and `EngineMemory`. `EngineSegment` is NOT wired into ISR
> replication yet (the fetch dispatch in `handle_fetch.rs` only matches
> Memory/RocksDB; the `_` arm returns `NotLeaderForPartition`). **Use
> `EngineRocksDB`** for this drill.

> **Timing — follower HW lags one fetch round.** A follower learns the leader's HW
> from the *next* fetch response, so its HW trails the leader by one round right
> after writes stop. Always **poll until convergence with a timeout** — never assert
> HW immediately after the last write.

## Prerequisites

- Build the actual executable (the `broker-server` binary is a `[[bin]]` in the
  **`cmd`** package, NOT the `broker-server` lib — `cargo build -p broker-server`
  silently builds only the library and leaves the binary stale):
  ```bash
  cargo build --bin broker-server
  ```
- Cluster configs: `config/cluster/server-{1,2,3}.toml`
  (broker_id 1/2/3; meta/grpc 1228/2228/3228; engine tcp 1779/2779/3779;
  admin http 58080/58082/58083). Data dirs: `data/broker-{1,2,3}`.
- The integration test already exists: `tests/tests/engine/isr.rs`
  (`three_replica_isr_sync`, `#[ignore]`). Test package name is **`robustmq-test`**.

## Drill Steps

### 0. Clean state (mandatory)

```bash
# Graceful stop of any leftover nodes (see the kill -INT note below), then wipe data.
for c in 1 2 3; do p=$(pgrep -f "server-$c.toml"); [ -n "$p" ] && kill -INT "$p"; done
for c in 1 2 3; do while pgrep -f "server-$c.toml" >/dev/null; do sleep 1; done; done
rm -rf data/broker-1 data/broker-2 data/broker-3 data/logs
```

> **NEVER `pkill -9` the broker on this machine.** SIGKILL leaves an `ESTABLISHED`
> socket on the grpc port (1228) that the local `mac_agent` keeps alive forever,
> blocking the next bind ("Failed to start GRPC server on port 1228: transport
> error"). Always stop with `kill -INT` (graceful). See Troubleshooting if 1228 is
> already stuck.

### 1. Start the 3-node cluster (staggered so node1 bootstraps first)

```bash
./target/debug/broker-server --conf config/cluster/server-1.toml > /tmp/n1.log 2>&1 &
sleep 8
./target/debug/broker-server --conf config/cluster/server-2.toml > /tmp/n2.log 2>&1 &
sleep 9
./target/debug/broker-server --conf config/cluster/server-3.toml > /tmp/n3.log 2>&1 &
sleep 13
```

Verify all three are up and registered:
```bash
grep -oE "bootstrapping single-node cluster|Successfully joined cluster via peer|Meta Service cluster is ready|Failed to start GRPC" /tmp/n1.log /tmp/n2.log /tmp/n3.log | sort | uniq -c
curl -s http://localhost:58080/api/info | python3 -c "import sys,json;d=json.load(sys.stdin).get('data',{});print('nodes:',sorted(n.get('node_id') for n in d.get('broker_node_list',[])))"
```
Expect node1 bootstrap, node2/3 joined, `nodes: [1, 2, 3]`. (A non-inner 3-replica
shard requires all 3 nodes alive — `create_shard` rejects if `alive < replica_num`.)

### 2. Run the integration test (the drill itself)

```bash
cargo test -p robustmq-test three_replica_isr_sync -- --ignored --nocapture
```

This test:
1. Creates a 3-replica `EngineRocksDB` shard via admin.
2. `get_shard` → asserts `replica_num == 3`, prints start/end offset + HW.
3. `segment_detail` → finds the leader, asserts `replicas.len() == 3`.
4. Writes 500 records (10×50) to the **leader** (discovered from `segment.leader`;
   the write client registers all 3 nodes' engine addresses).
5. Polls `segment_detail` until every replica is `available`, `in_isr`,
   `leo == leader_leo`, **and `high_watermark == leader_leo`** (timeout 30s).
6. Final asserts: ISR == 3, every replica leo/hw == leader_leo, leader hw == leo,
   `get_shard` HW == 500, end_offset == 499.

Expected passing output (shape):
```
get_shard ...: replica_num=3 start_offset=0 end_offset=0 high_watermark=0
shard ... seg 0: leader=n3 replicas=[3, 2, 1] isr=[1, 2, 3]
[2.9ms]  leader_leo=500 isr=3 | n3(leo=500,hw=500) n2(leo=500,hw=350) n1(leo=500,hw=350)
[508ms]  leader_leo=500 isr=3 | n3(leo=500,hw=500) n2(leo=500,hw=500) n1(leo=500,hw=500)
ISR converged: 3 replicas all at LEO=HW=500
get_shard ... after writes: start_offset=0 end_offset=499 high_watermark=500
test result: ok. 1 passed
```
Note how follower HW starts behind (350) and catches up to 500 — that lag is normal.

### 3. (Optional) Manual inspection via admin API

If you want to watch sync live without the test, after creating a shard:
```bash
# segment list → active segment_seq, then segment detail (per-replica leo/hw/isr)
curl -s -X POST http://localhost:58080/api/storage-engine/segment/detail \
  -H 'Content-Type: application/json' -d '{"shard_name":"<shard>","segment_seq":0}' | python3 -m json.tool
# shard view (start/end offset + high_watermark)
curl -s -X POST http://localhost:58080/api/storage-engine/shard/list \
  -H 'Content-Type: application/json' -d '{"shard_name":"<shard>"}' | python3 -m json.tool
```
`segment_detail` requires `segment_seq` — first get it from the segment list
(`active_segment_seq`); it does not auto-pick the active segment.

## Drill B — kill leader failover + rejoin

Verifies the cluster survives losing the segment leader **without data loss** and
that the killed node rejoins the ISR after restart.

```bash
cargo test -p robustmq-test three_replica_leader_failover -- --ignored --nocapture
```

What it does:
1. Creates a 3-replica `EngineRocksDB` shard; writes 100 records to the leader
   with **acks=all** (so they are committed on all ISR replicas — `leo=hw=100`).
2. Confirms all 3 replicas show `leo=hw=100`, then **kills the leader broker
   process** (`pkill -INT -f server-N.toml`).
3. Polls a surviving node's admin until `segment.leader` changes; asserts the new
   leader is a surviving replica, `leader_epoch` bumped, the dead node left the ISR
   but is **retained in `replicas`** (temporary-offline).
4. **No data loss**: polls until both survivors settle at `leo=hw=100`, `lso=0`
   (committed data preserved across the switch).
5. Writes 100 more to the **new** leader (acks=all) → survivors converge to
   `leo=hw=200`.
6. Restarts the killed node and asserts it **rejoins the ISR** and catches up
   (`leo=hw=200`, `lso=0`).

Expected passing output (shape):
```
all 3 replicas committed LEO=HW=100
LEADER SWITCHED n2 -> n1 after 28.1s (epoch 0 -> 1, isr=[1], replicas=[2, 3, 1])
no data loss: both survivors hold committed LEO=HW=100
survivors converged: 2 live replicas at LEO=HW=200
node2 REJOINED ISR after 4.0s: isr=[1, 2, 3]
test result: ok. 1 passed
```

Key behaviours and timings (verified):
- **Switch latency ≈ 28–30s**, driven by the meta-service heartbeat timeout
  (`heartbeat_timeout_ms`, default 30s). A graceful `kill -INT` does NOT shorten it
  — the broker does not actively unregister on shutdown, so `kill -INT` and
  `kill -9` both wait for the heartbeat timeout.
- **acks=all is required** to prove no-loss: it carries a producer commit timeout
  (`WriteReqBody.timeout_ms`, default 30s) and returns only once the HW reaches the
  written offset on all ISR replicas. With acks=1 the un-committed tail is lost on
  failover (expected Kafka-like semantics, not a bug).
- **Rejoin** is driven by the metadata reconcile thread (~30s interval) self-healing
  a follower that has no fetcher; look for the WARN
  `reconcile: follower has no fetcher for <shard>/<seg> (leader=N), starting replication`.

This drill guards three fixes — a regression in any of them fails it:
| Fix | Bug it prevents |
|-----|-----------------|
| `dynamic_cache.rs` Create handler only inits offsets for a genuinely-new segment (`get_offset_state().is_none()`) | leader-switch "Create" notification resetting `latest_offset→0` on a node that already holds the data → committed data truncated/lost |
| `WriteReqBody.timeout_ms` (producer commit timeout, threaded into `batch_write`'s acks=all wait) | acks=all spuriously timing out because it reused the 500ms `replica_fetch_max_wait_ms` |
| `reconcile.rs` self-heal: start a fetcher when this node is a follower with no `fetch_state` | a restarted follower that missed the leader-switch notification never resuming replication → never rejoining the ISR |

> **Process restart from a test:** `cargo test` runs with CWD = the test package
> dir, NOT the repo root. The test restarts the killed broker via the repo root
> (resolved from `CARGO_MANIFEST_DIR`) with `current_dir(repo_root)`. A bare
> `./target/debug/broker-server` would silently fail to spawn.

## Drill C — rolling-restart chaos (frequent leader switches)

Stresses the ISR machinery under a sustained rolling restart: 10 rounds, each
restarts ONE node (rotating 1→2→3→1…) so the meta-service Raft always keeps a 2/3
quorum (killing two would lose it — don't). Restarting the current leader forces a
switch, so leadership moves around frequently; the test asserts the ISR stays
correct throughout.

```bash
cargo test -p robustmq-test three_replica_chaos_rolling_kill -- --ignored --nocapture
```

Each round: write a committed batch (acks=all) → kill one node → wait for it to
leave the ISR (and leadership to move if it was the leader) → write another batch
on the 2 survivors (acks=all, still committed) → restart the node → wait for it to
rejoin and catch up the writes it missed → verify. Per round it checks, on EVERY
segment-detail read:

- `hw <= leo` invariant (HW/offset-bookkeeping corruption)
- `replicas == {1,2,3}` always; dead node leaves ISR then ISR recovers to `{1,2,3}`
- every replica `leo == hw == cumulative`, `lso == 0` (no loss, full catch-up)
- `leader_epoch` monotonic (fencing); **all 3 nodes agree on leader/epoch/ISR**
  (split-brain / stale-view check)
- acks=all commits in both full and degraded (2-node) ISR (no write stall)

and at the end reads back all `cumulative` records and scans every node log for the
blacklist (see Log Analysis). Expected tail:

```
round 1 OK: leader=n3 epoch=0 isr=[1, 2, 3] all leo=hw=60 lso=0
...
round 10 OK: leader=n3 epoch=6 isr=[1, 2, 3] all leo=hw=600 lso=0
CHAOS COMPLETE: 10 rounds, 600 committed records read back OK, 6 leader-epoch switches, all replicas consistent
```

Note the timings the drill exercises: a dead **follower** leaves the ISR in ~6-11s
(leader-side lag shrink, `replica_lag_time_max_ms`), a dead **leader** in ~28-30s
(heartbeat timeout → switch). This drill found and guards three real ISR defects:

| Fix | Bug it prevents |
|-----|-----------------|
| `broker-core` heartbeat reports to **every** meta node, not one | heartbeat table is in-memory per meta node + the expiry check is leader-only; after a meta-leader change the new leader never saw heartbeats and expired ALL nodes every `heartbeat_timeout_ms` → endless cluster-wide leader/ISR churn that never self-heals |
| `isr_manager::compute_new_isr` keeps a follower in-sync purely by fetch **recency** (`last_fetch_ts`), not a static `leo >= leader_leo` | a follower that died while fully caught up kept `leo == leader_leo` against a non-advancing leader LEO forever → dead replica pinned in the ISR indefinitely (blocks acks=all, stale ISR) |
| `dynamic_cache.rs` Create handler also `remove_leader_segment` when this node is NOT the new leader | leader switch is a "Create" notification; a demoted node kept the segment in `leader_segments` and kept running ISR maintenance for it → stale ISR proposals / divergent maintenance view across nodes |
| `isr/apply.rs::apply_as_follower` passes `leader_epoch_changed` flag — only sets `needs_truncation: true` when the leader epoch actually changed, not on every Segment Update | every ISR membership notification (e.g. ISR shrink when a follower was killed) called `apply_as_follower` unconditionally with `needs_truncation: true`, stopping the fetcher for a full OffsetsForLeaderEpoch round-trip; when this gap overlapped an acks=all write's 30s window it raced to a TokioTimeErrorElapsed timeout |

## Pass / Fail Criteria

PASS only if **all** hold after convergence (within the 30s poll window):
- `replica_num == 3`, `segment.replicas.len() == 3`, `isr` has all 3 nodes.
- Every replica: `available == true`, `in_isr == true`, `leo == leader_leo`,
  `high_watermark == leader_leo`.
- `leader.high_watermark == leader.leo`; `leader_leo == records_written` (500).
- `get_shard.high_watermark == 500`, `end_offset == 499`.

Common FAIL signatures and what they mean:
| Symptom | Likely cause |
|---------|--------------|
| follower `leo` never reaches leader `leo` | replication pipe broken (fetcher thread / transport) |
| follower `hw` stuck at 0 while `leo` caught up | follower not applying `leader_hw` (the fix in `fetcher.rs::apply_shard_resp`) |
| a replica `available=false` + `Invalid URL` | segment-detail fan-out built http_addr without scheme |
| `isr` shrinks below 3 | a replica fell behind / heartbeat lost |
| `create_shard` fails "not enough nodes" | a node is down — non-inner 3-replica needs 3 alive |

## Log Analysis

```bash
for c in 1 2 3; do echo "node$c ERROR: $(grep -c ' ERROR' /tmp/n$c.log)"; done   # expect 0
```
Transient `Unreachable node` WARNs only on the leader around a kill are benign.

## Cleanup

```bash
# Graceful stop — do NOT pkill -9 (leaves a stuck 1228 socket via mac_agent).
for c in 1 2 3; do p=$(pgrep -f "server-$c.toml"); [ -n "$p" ] && kill -INT "$p"; done
for c in 1 2 3; do while pgrep -f "server-$c.toml" >/dev/null; do sleep 1; done; done
```

## Troubleshooting

| Symptom | Cause / fix |
|---------|-------------|
| node1 `Failed to start GRPC server on port 1228: transport error`, but `lsof -iTCP:1228 -sTCP:LISTEN` shows nothing | A `pkill -9` left an `ESTABLISHED` orphan on local port 1228 that `mac_agent` (`/usr/local/bin/mac_agent`, connects to every localhost port) keeps alive. Check `netstat -anp tcp \| grep '\.1228 '` — if you see `127.0.0.1.1228 ... ESTABLISHED` with no owning broker process, the only clean fixes are: restart/`kill` the `mac_agent` pid (launchd respawns it) to release the socket, or reboot. **Prevent it by always stopping with `kill -INT`.** |
| Rebuilt code "didn't take" / binary mtime unchanged | `cargo build -p broker-server` builds only the lib. Use `cargo build --bin broker-server`. Verify: `strings target/debug/broker-server \| grep <a string you changed>`. |
| Test fails because follower HW lags the leader | Don't assert HW immediately — poll until `hw == leader_leo` with a timeout. The follower learns HW one fetch round later. |
| Used `EngineSegment` and replicas never sync | ISR replication is not implemented for `EngineSegment`. Use `EngineRocksDB`/`EngineMemory`. |
| `segment_detail` returns null `data` | The segment isn't in cache yet — wait for provisioning (~10s) and confirm `segment_seq` exists via the segment list. |

## Run Report (generate after each complete A+B+C session)

After all three drills finish (or fail), emit a structured summary so the results
are recorded in the conversation. Collect the key facts from test output and logs,
then print a report in this format:

```
=== ISR REPLICATION DRILL RUN REPORT ===
Date:        <YYYY-MM-DD>
Code commit: <git rev-parse --short HEAD>

Drill A — replica sync
  Result:    PASS / FAIL
  Details:   ISR converged at LEO=HW=<N> in <T>s

Drill B — leader failover
  Result:    PASS / FAIL
  Details:   leader switched n<X>->n<Y> after <T>s, no data loss, rejoin after <T>s

Drill C — rolling-restart chaos (10 rounds)
  Result:    PASS / FAIL
  Rounds:    <N>/10 passed
  Records:   <N> committed records read back OK
  Epochs:    <N> leader-epoch switches
  Details:   <any notable anomalies or the failure round + panic line>

Log errors:
  node1: <N> ERRORs
  node2: <N> ERRORs
  node3: <N> ERRORs

Overall:  PASS / FAIL
========================================
```

Gather inputs with:
```bash
git rev-parse --short HEAD
# Drill results come from the test output captured above.
for c in 1 2 3; do echo "node$c ERROR: $(grep -c ' ERROR' /tmp/n$c.log)"; done
```
