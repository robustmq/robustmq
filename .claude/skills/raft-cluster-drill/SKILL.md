---
name: raft-cluster-drill
description: Run a 3-node meta-service Raft cluster failover drill for RobustMQ. Use when the user wants to verify cluster startup, node removal/recovery, leader election, snapshot replication, or graceful shutdown — e.g. "演练 raft 集群", "test cluster failover", "验证节点删除/恢复", "run the raft drill".
---

# RobustMQ Raft Cluster Drill

A reproducible drill to verify the meta-service Raft cluster behaves correctly
on RobustMQ's openraft-based multi-runtime broker. It exercises bootstrap, join,
leader election, node recovery, snapshot replication, and graceful shutdown.

## When to Use

- Verify a fresh 3-node cluster starts and forms membership `{1,2,3}`
- Confirm killing a node does NOT change membership and (if it was leader) triggers re-election
- Confirm restarting a node recovers automatically (no re-join needed)
- Confirm `Ctrl+C` / `kill -INT` exits each node gracefully
- Regression-check the openraft fixes: signal handling, snapshot send/receive, no leave-on-stop

## Prerequisites

- Build the binary first (drills must run the compiled binary, NOT `cargo run` —
  `cargo run` intercepts SIGINT via its parent process and masks real shutdown behavior):
  ```bash
  cargo build --package cmd --bin broker-server
  ```
- Cluster configs live in `config/cluster/server-{1,2,3}.toml`
  (broker_id 1/2/3, grpc 1228/2228/3228, admin http 58080/58082/58083,
  meta_addrs = all three). Data dirs: `data/broker-{1,2,3}`.

## Key Semantics (openraft) — what "correct" looks like

| Action | Expected behavior |
|--------|-------------------|
| First node, no peer reachable | single-node `bootstrap` → becomes Leader |
| Fresh node, peer reachable | `join` (add_learner + change_membership), snapshot streamed if lagging |
| Node restart (has persisted state) | `recover` — openraft re-establishes replication, **no** re-join, membership unchanged |
| Stop a node | membership **unchanged** `{1,2,3}` (stop ≠ scale-in); if it was leader, remaining quorum re-elects |
| Graceful stop | SIGINT/SIGTERM → optional drain → process exits (libc handler + watchdog) |

Quorum rule: 3 nodes → need ≥2 alive to elect/serve. Never kill down to <quorum
while a node is still starting, or that node hangs in `check_meta_service_status`.

## Drill Steps

### 0. Always clean state first (mandatory)

Every drill run starts from a clean slate — kill any leftover process and wipe
all node data. Stale Raft state from a previous run will otherwise interfere
(wrong membership, leftover leader, etc.):

```bash
pkill -9 -f "broker-server --conf" 2>/dev/null; sleep 1
rm -rf data/broker-1 data/broker-2 data/broker-3 data/logs
```

### 1. Start the 3-node cluster (staggered so node1 bootstraps first)

```bash
./target/debug/broker-server --conf config/cluster/server-1.toml > /tmp/n1.log 2>&1 &
sleep 7
./target/debug/broker-server --conf config/cluster/server-2.toml > /tmp/n2.log 2>&1 &
sleep 9
./target/debug/broker-server --conf config/cluster/server-3.toml > /tmp/n3.log 2>&1 &
sleep 12
```

Verify in logs:
- node1: `No reachable peers, bootstrapping single-node cluster (node 1)`
- node2/node3: `Successfully joined cluster via peer 127.0.0.1:1228` (NO `snapshot not found`)

### 2. Query Raft state via admin API (`/api/info`)

Each admin port returns `data.meta` with the three state machines
(`metadata_0`, `offset_0`, `data_0`). Cross-check all nodes agree:

```bash
for p in 58080 58082 58083; do
  echo "--- port $p ---"
  curl -s http://localhost:$p/api/info | python3 -c "
import sys,json
d=json.load(sys.stdin).get('data',{})
for n,rs in sorted(d.get('meta',{}).items()):
    cfg=rs.get('membership_config',{}).get('membership',{}).get('configs',[])
    print(f'  [{n}] state={rs.get(\"state\")} leader={rs.get(\"current_leader\")} term={rs.get(\"current_term\")} members={cfg}')
"
done
```

Expect: all three nodes report the same leader/term per shard, members `[[1,2,3]]`.

### 3. Drill A — remove a node (membership + leader check)

Kill a node, wait for graceful exit, re-query:

```bash
kill -INT $(pgrep -f "server-3.toml")   # or server-1 to test leader failover
# wait until the process exits, then re-run step 2
```

Expect:
- Killed node exits gracefully in ~5–7s.
- Membership stays `[[1,2,3]]` (stop does not remove the node).
- If you killed the **leader**, remaining nodes re-elect (new `current_leader`, higher `term`).
- If you killed a **follower**, the leader is unchanged.

### 4. Drill B — rejoin the node (recovery)

Restart the killed node; it has persisted state so it recovers:

```bash
./target/debug/broker-server --conf config/cluster/server-3.toml > /tmp/n3.log 2>&1 &
sleep 12
```

Expect in log: `Node 3 has persisted state, recovering existing cluster` and
`Meta Service cluster is ready`. Re-run step 2 — all nodes back, consensus consistent.

### 5. Drill D — rolling kill + restart of every node (failover stress)

Kill each node in turn, observe leader failover, then restart it and observe it
rejoin — going around all three nodes. **Always keep ≥2 nodes alive** (kill one,
restart it before killing the next), otherwise quorum is lost.

```bash
for c in 1 2 3; do
  echo "===== rolling: node$c ====="
  # leader before kill (ask a node that stays up)
  ask=$([ "$c" = 1 ] && echo 58082 || echo 58080)
  echo "leader before: $(curl -s http://localhost:$ask/api/info | python3 -c "import sys,json;m=json.load(sys.stdin).get('data',{}).get('meta',{});print({n:rs.get('current_leader') for n,rs in m.items()})")"

  # kill node$c, wait for graceful exit
  kill -INT $(pgrep -f "server-$c.toml")
  while pgrep -f "server-$c.toml" >/dev/null; do sleep 1; done
  sleep 4
  echo "leader after kill: $(curl -s http://localhost:$ask/api/info | python3 -c "import sys,json;m=json.load(sys.stdin).get('data',{}).get('meta',{});print({n:rs.get('current_leader') for n,rs in m.items()})")"

  # restart node$c, wait to rejoin
  ./target/debug/broker-server --conf config/cluster/server-$c.toml > /tmp/n$c.log 2>&1 &
  sleep 14
  grep -oE "recovering existing cluster|Meta Service cluster is ready" /tmp/n$c.log | sort -u
  echo "members after rejoin: $(curl -s http://localhost:$ask/api/info | python3 -c "import sys,json;m=json.load(sys.stdin).get('data',{}).get('meta',{});print([rs.get('membership_config',{}).get('membership',{}).get('configs',[]) for rs in m.values()][:1])")"
done
```

Expect each iteration:
- Killing a **leader** → remaining quorum re-elects (leader changes, term +1).
  Killing a **follower** → leader unchanged.
- Membership stays `[[1,2,3]]` throughout (kill ≠ scale-in).
- Restarted node logs `recovering existing cluster` + `Meta Service cluster is ready`,
  rejoins automatically, all nodes converge on the same leader/term.

### 6. Drill C — graceful shutdown

Kill nodes one by one, each waiting for the previous to exit
(so quorum is never lost while a node is still starting):

```bash
for c in 3 2 1; do
  kill -INT $(pgrep -f "server-$c.toml")
  while pgrep -f "server-$c.toml" >/dev/null; do sleep 1; done
  echo "node$c exited"
done
```

Expect: each node prints `Termination signal received, the service starts to stop`
then exits. The leader/data-leader node may take longer (waits on replication to
downed peers) but still exits within the watchdog window.

## Log Analysis — run after every drill step

After each step, scan all node logs for ERROR/WARN and judge whether each is
expected. This is how a real regression was caught (a busy-loop that left a node
at 99% CPU while the cluster still looked "healthy").

```bash
# ERROR count per node — should be 0
for c in 1 2 3; do echo "node$c ERROR: $(grep -c ERROR /tmp/n$c.log 2>/dev/null)"; done

# WARN types per node
for c in 1 2 3; do
  echo "--- node$c WARN ---"
  grep WARN /tmp/n$c.log 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -oE "WARN [a-z_:]+" | sort | uniq -c | sort -rn
done

# broker_node_list must list all live nodes (a missing live node = heartbeat/registration bug)
curl -s http://localhost:58080/api/info | python3 -c "import sys,json;d=json.load(sys.stdin).get('data',{});print('node_ids:', sorted(n.get('node_id') for n in d.get('broker_node_list',[])))"

# CPU per node — a node stuck near 100% = busy loop (RN state), not normal
for c in 1 2 3; do P=$(pgrep -f "server-$c.toml"); [ -n "$P" ] && ps -p $P -o pid,stat,%cpu | tail -1; done
```

### Which WARNs are benign vs. a real problem

| Log | Verdict |
|-----|---------|
| `openraft::core::raft_core: membership_log_id changed ... ignore` | **Benign** — concurrent membership change dedup during join |
| `meta_service::raft::manager: Peer N not reachable: Connection refused` (at startup) | **Benign** — peers not up yet; first node bootstraps |
| `mqtt_broker::system_topic: Failed to write ... Connection refused` that **keeps growing** | **PROBLEM** — a system-topic shard replica points at an unreachable/removed node. Check `broker_node_list` for a missing live node. |
| Any `ERROR` | **PROBLEM** — investigate. |
| A node at ~99% CPU / `RN` state | **PROBLEM** — busy loop (e.g. retry_call spinning). |
| `broker_node_list` missing a live node | **PROBLEM** — that node's heartbeat/registration failed (often a knock-on effect of a busy loop). |

## Cleanup

```bash
pkill -9 -f "broker-server --conf"
```

## Troubleshooting

| Symptom | Cause |
|---------|-------|
| `cargo run` + Ctrl+C exits instantly with no shutdown logs | Expected — cargo kills the child. Use the compiled binary. |
| Node hangs in `Waiting for Meta Service cluster to be ready` | Quorum lost (too many nodes down/starting). Keep ≥2 nodes healthy. |
| `snapshot not found` on join | Regression in `get_current_snapshot` / `begin_receiving_snapshot` (state.rs). |
| Process won't exit on `kill -INT`, only `kill -9` works | Regression in signal handling (libc handler in daemon.rs) or a `leave_cluster` re-introduced into shutdown. |
