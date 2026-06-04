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
- Verify **state-machine data correctness** under repeated kill/restart: data written
  through Raft survives restarts, all 3 nodes converge, `last_applied` is monotonic
  (write→restart→read-back; Drill F)
- Regression-check the openraft fixes: signal handling, snapshot send/receive,
  no leave-on-stop, total_order_seek prefix reads

## Prerequisites

- Build the binary first (drills must run the compiled binary, NOT `cargo run` —
  `cargo run` intercepts SIGINT via its parent process and masks real shutdown behavior):
  ```bash
  cargo build --package cmd --bin broker-server
  ```
- Cluster configs live in `config/cluster/server-{1,2,3}.toml`
  (broker_id 1/2/3, grpc 1228/2228/3228, admin http 58080/58082/58083,
  meta_addrs = all three). Data dirs: `data/broker-{1,2,3}`.
- Admin list endpoints (`/api/cluster/*/list`) default to `limit=10`. When a drill
  accumulates >10 rows, ALWAYS pass an explicit `?limit=100000`, or the
  lexicographically-last rows are paginated out and look "missing". Trust
  `total_count` over the returned row count.
- Run drill scripts with **bash, not zsh** (zsh's 1-based arrays + no `$var`
  word-splitting silently break the kill order and port lookups).

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
    la=(rs.get('last_applied') or {}).get('index')
    print(f'  [{n}] state={rs.get(\"state\")} leader={rs.get(\"current_leader\")} term={rs.get(\"current_term\")} last_log_index={rs.get(\"last_log_index\")} last_applied={la} members={cfg}')
"
done
```

Expect: all three nodes report the same leader/term per shard, members `[[1,2,3]]`,
and identical `last_applied` / `last_log_index` per shard (consensus converged).

> Drills A–F are historical names; run them in the section order below
> (A remove → B rejoin → D rolling → E sustained stress → C graceful shutdown →
> F state-machine correctness). Each is self-contained given a running cluster.

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

### 5b. Drill E — sustained rolling kill+restart stress (graceful-exit watch)

A longer stress variant of Drill D focused on **graceful shutdown under churn**:
across many rounds, kill+restart **every** node, plus one extra operation per
round (so each round performs **4 kill+restart cycles**, e.g. nodes
`1 → 2 → 3 → 1`). The wrap-around (operating node 1 again) deliberately hits a
node that was *just* restarted in the same round, exercising
kill-during-recovery. **Always keep ≥2 nodes alive** — restart each node and
let it fully rejoin before killing the next.

For each kill, this drill asserts the node **exits gracefully** (prints
`Termination signal received` and exits on its own within the watchdog window —
NO `kill -9` fallback, NO 20s force-exit). For each restart, it asserts the node
**rejoins** (`recovering existing cluster` + `Meta Service cluster is ready`).

```bash
GRACE_TIMEOUT=25   # seconds to wait for self-exit before declaring a hang

graceful_kill() {  # $1 = node id
  local c=$1 t0
  kill -INT "$(pgrep -f "server-$c.toml")"
  t0=$(date +%s)
  while pgrep -f "server-$c.toml" >/dev/null; do
    sleep 1
    if [ $(($(date +%s) - t0)) -gt "$GRACE_TIMEOUT" ]; then
      echo "  !!! node$c DID NOT EXIT in ${GRACE_TIMEOUT}s — HANG, forcing kill -9"
      kill -9 "$(pgrep -f "server-$c.toml")"; return 1
    fi
  done
  local secs=$(($(date +%s) - t0))
  # graceful = printed the shutdown banner AND did not hit the watchdog force-exit
  if grep -q "Termination signal received" /tmp/n$c.log && \
     ! grep -q "forcing exit" /tmp/n$c.log; then
    echo "  node$c exited GRACEFULLY in ${secs}s"
  else
    echo "  !!! node$c exit was NOT graceful (no banner or watchdog force-exit) in ${secs}s"; return 1
  fi
}

restart_node() {  # $1 = node id
  local c=$1
  ./target/debug/broker-server --conf config/cluster/server-$c.toml > /tmp/n$c.log 2>&1 &
  sleep 14
  if grep -q "Meta Service cluster is ready" /tmp/n$c.log; then
    echo "  node$c REJOINED ($(grep -oE 'recovering existing cluster' /tmp/n$c.log | head -1 | sed 's/.*/recovering/'))"
  else
    echo "  !!! node$c did NOT report cluster ready after restart"
  fi
}

for round in $(seq 1 10); do
  echo "========== Drill E round $round =========="
  for c in 1 2 3 1; do          # 3 nodes + 1 wrap-around = 4 ops/round
    echo "--- op: node$c (kill+restart) ---"
    graceful_kill "$c"
    restart_node "$c"
  done
  # consensus + error scan after each round
  ask=58080
  echo "members: $(curl -s http://localhost:$ask/api/info | python3 -c "import sys,json;m=json.load(sys.stdin).get('data',{}).get('meta',{});print([rs.get('membership_config',{}).get('membership',{}).get('configs',[]) for rs in m.values()][:1])" 2>/dev/null)"
  PANIC=$(( $(grep -c 'invalid state: expect' /tmp/n1.log)+$(grep -c 'invalid state: expect' /tmp/n2.log)+$(grep -c 'invalid state: expect' /tmp/n3.log) ))
  NONE=$(( $(grep -c 'last_log_id=None' /tmp/n1.log)+$(grep -c 'last_log_id=None' /tmp/n2.log)+$(grep -c 'last_log_id=None' /tmp/n3.log) ))
  ERR=$(( $(grep -c ' ERROR' /tmp/n1.log)+$(grep -c ' ERROR' /tmp/n2.log)+$(grep -c ' ERROR' /tmp/n3.log) ))
  FORCE=$(( $(grep -c 'forcing exit' /tmp/n1.log)+$(grep -c 'forcing exit' /tmp/n2.log)+$(grep -c 'forcing exit' /tmp/n3.log) ))
  echo "round $round: raft_panic=$PANIC last_log_id_None=$NONE ERROR=$ERR watchdog_force_exit=$FORCE"
done
```

Watch for (any of these = a problem to investigate):
- **Hang**: a node not exiting within `GRACE_TIMEOUT` on `kill -INT` (only `kill -9` works).
- **Non-graceful exit**: watchdog `forcing exit` fired, or no `Termination signal received` banner.
- **`raft_panic` / `last_log_id_None` / `clean_hole` > 0**: the storage regression resurfaced.
- **Membership drifted** off `[[1,2,3]]`, or a node failed to report `Meta Service cluster is ready` after restart.
- **`ERROR` lines** that are not the known-benign startup `Connection refused` (peer still starting).

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

### 7. Drill F — state-machine correctness under repeated kill/restart

This is the deepest drill: it verifies the **state machine itself stays correct**
across repeated kill/restart, not just that nodes come back up. Each round writes
data through Raft, kills+restarts every node (leader first, on purpose), then
proves the data survived and the three nodes agree.

Verifies four properties:
1. **Write→restart→read-back** — data committed before a restart is still readable
   on every node afterward (the restarted node must rebuild its cache from the
   persisted state machine). Uses tenants: `POST /api/cluster/tenant/create`
   (Raft-committed, auto-forwarded to leader) and `GET /api/cluster/tenant/list`
   (served from each node's local cache, which Raft apply maintains). No auth.
2. **Three-node consensus convergence** — every node's `data.meta.<group>`
   reports the same `last_applied.index` / `current_term` / `current_leader` /
   members per Raft group.
3. **`last_applied` monotonicity** — `last_applied.index` per group never goes
   backward across rounds (apply progress is durably persisted).
4. **Targeted leader kill** — each round kills the *current metadata leader*
   first, so failover + graceful-exit-of-leader is exercised every round.

> **Run with bash, NOT zsh.** Save as a script with `#!/bin/bash` and run
> `bash drill_f.sh`. zsh's 1-based arrays and lack of `$var` word-splitting
> silently break the port lookup and the kill order (the cluster never gets
> killed and writes go to an empty port) — the drill then *looks* like it passes
> while doing nothing.

```bash
#!/bin/bash
set -u
GRACE_TIMEOUT=25

port_of() { case "$1" in 1) echo 58080;; 2) echo 58082;; 3) echo 58083;; esac; }

graceful_kill() {  # $1 = node id; asserts self-exit + graceful, no kill -9
  local c=$1 t0 pid; pid=$(pgrep -f "server-$c.toml")
  [ -z "$pid" ] && { echo "  !!! node$c not running before kill"; return 1; }
  kill -INT "$pid"; t0=$(date +%s)
  while pgrep -f "server-$c.toml" >/dev/null; do sleep 1
    [ $(($(date +%s)-t0)) -gt "$GRACE_TIMEOUT" ] && { echo "  !!! node$c HANG -> kill -9"; kill -9 "$(pgrep -f "server-$c.toml")"; return 1; }
  done
  grep -q "Termination signal received" /tmp/n$c.log && ! grep -q "forcing exit" /tmp/n$c.log \
    && echo "  node$c GRACEFUL ($(($(date +%s)-t0))s)" || { echo "  !!! node$c NOT graceful"; return 1; }
}
restart_node() { ./target/debug/broker-server --conf config/cluster/server-$1.toml > /tmp/n$1.log 2>&1 & sleep 14; }

# find the metadata leader node id by asking any live node
meta_leader() {
  local p L
  for p in 58080 58082 58083; do
    L=$(curl -s "http://127.0.0.1:$p/api/info" | python3 -c "import sys,json;m=json.load(sys.stdin).get('data',{}).get('meta',{});print(m.get('metadata_0',{}).get('current_leader') or '')" 2>/dev/null)
    [ -n "$L" ] && { echo "$L"; return; }
  done
}

EXPECT=()                 # accumulated tenant names that must always read back

for round in $(seq 1 10); do
  echo "========== Drill F round $round =========="

  # (1) WRITE a unique tenant through Raft, then remember it
  TN="drillf-r${round}"
  RESP=$(curl -s -X POST "http://127.0.0.1:$(port_of 1)/api/cluster/tenant/create" \
    -H 'Content-Type: application/json' -d "{\"tenant_name\":\"$TN\",\"desc\":\"r$round\"}")
  CODE=$(echo "$RESP" | python3 -c "import sys,json;print(json.load(sys.stdin).get('code'))" 2>/dev/null)
  [ "$CODE" = "0" ] && { EXPECT+=("$TN"); echo "  wrote tenant $TN (code=0)"; } || echo "  !!! write $TN failed: $RESP"

  # (4) kill the LEADER first, then the other two — build a SPACE-separated order
  LEADER=$(meta_leader); echo "  metadata leader = node$LEADER (killed first)"
  ORDER="$LEADER"
  for n in 1 2 3; do case " $ORDER " in *" $n "*) ;; *) ORDER="$ORDER $n";; esac; done
  for c in $ORDER; do
    echo "--- op: node$c ---"; graceful_kill "$c"; restart_node "$c"
  done

  # let caches/consensus settle after the last restart
  sleep 4

  # (2)+(3) consensus convergence + monotonicity, read from all 3 nodes
  python3 - "$round" <<'PY'
import sys,json,urllib.request
rnd=sys.argv[1]; ports=[58080,58082,58083]; snaps={}
for p in ports:
    try: snaps[p]=json.load(urllib.request.urlopen(f"http://127.0.0.1:{p}/api/info",timeout=5)).get('data',{}).get('meta',{})
    except Exception as e: snaps[p]=None; print(f"  !!! node@{p} /api/info failed: {e}")
groups=sorted({g for m in snaps.values() if m for g in m})
ok=True
for g in groups:
    rows={p:(m.get(g) if m else None) for p,m in snaps.items()}
    la={p:(r.get('last_applied') or {}).get('index') for p,r in rows.items() if r}
    tm={p:r.get('current_term') for p,r in rows.items() if r}
    ld={p:r.get('current_leader') for p,r in rows.items() if r}
    mb={p:r.get('membership_config',{}).get('membership',{}).get('configs') for p,r in rows.items() if r}
    conv = len(set(la.values()))<=1 and len(set(tm.values()))<=1 and len(set(ld.values()))<=1
    print(f"  [{g}] last_applied={la} term={tm} leader={ld} members={list(mb.values())[0] if mb else None} converged={'YES' if conv else 'NO <<<'}")
    if not conv: ok=False
print("  CONSENSUS:", "CONVERGED" if ok else "DIVERGED <<<<<")
PY

  # (1) read-back: every accumulated tenant must be present on ALL 3 nodes.
  # NOTE: pass a large limit — list endpoints default to limit=10 (see
  # parse_limit in admin-server/src/tool/query.rs). Without it, the 11th+ row
  # (e.g. the lexicographically-last "drillf-r9" among r1..r10) is silently
  # paginated out and read-back falsely reports it MISSING.
  MISS=0
  for p in 58080 58082 58083; do
    HAVE=$(curl -s "http://127.0.0.1:$p/api/cluster/tenant/list?limit=100000" | python3 -c "import sys,json;print(' '.join(r['tenant_name'] for r in json.load(sys.stdin).get('data',{}).get('data',[])))" 2>/dev/null)
    for t in "${EXPECT[@]}"; do case " $HAVE " in *" $t "*) ;; *) echo "  !!! node@$p MISSING tenant $t"; MISS=$((MISS+1));; esac; done
  done
  [ "$MISS" = 0 ] && echo "  READ-BACK: all ${#EXPECT[@]} tenants present on all 3 nodes" || echo "  READ-BACK: $MISS MISSING <<<<<"

  # error/panic scan for the round
  PANIC=$(( $(grep -c 'invalid state: expect' /tmp/n1.log)+$(grep -c 'invalid state: expect' /tmp/n2.log)+$(grep -c 'invalid state: expect' /tmp/n3.log) ))
  NONE=$(( $(grep -c 'last_log_id=None' /tmp/n1.log)+$(grep -c 'last_log_id=None' /tmp/n2.log)+$(grep -c 'last_log_id=None' /tmp/n3.log) ))
  echo "  >>> round $round: raft_panic=$PANIC last_log_id_None=$NONE"
done
```

Expect every round: `CONSENSUS: CONVERGED`, `READ-BACK: all N tenants present`,
`last_applied` non-decreasing per group, leader killed exits gracefully, and
new leader elected. Then run the WARN/ERROR analysis below.

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
| `openraft::replication: ... Unreachable node` ERROR/WARN (+ `heartbeat error` / `error replication to target`) **clustered around a kill** | **Benign & transient** — the leader briefly fails to replicate/heartbeat to the node you just killed; stops the moment that node restarts. Expected to appear only on the current leader, only at kill times, and NOT keep growing. A real problem only if it persists after the target is back up, or targets a node that is actually alive. |
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
| Nodes fail to start after a few restart rounds; `RaftCore exited ... invalid state: purge_upto > snapshot_last_log_id` | Logs are read back empty on restart. Confirm by grepping the startup logs for `get_initial_state ... last_log_id=None` and `Clean the hole`. Root cause was `get_log_state`/`try_get_log_entries` iterating without `total_order_seek` while the DB has a 10-byte fixed-prefix extractor (log keys are 17 bytes) — the reverse seek finds nothing and openraft thinks the whole log was lost, then purges past the snapshot. Fixed in `raft/store/log.rs` by setting `ReadOptions::set_total_order_seek(true)` on both iterators (regression test: `test_get_log_state_survives_reopen_with_prefix_extractor`). |
| Drill F read-back reports a tenant MISSING but point-get / `list?limit=big` / `total_count` show it IS present | **Test artifact, NOT a bug.** List endpoints default to `limit=10` (`parse_limit`, admin-server/src/tool/query.rs). With >10 rows the lexicographically-last name is paginated out. Always query lists with an explicit large `?limit=`. Verify with `/api/cluster/tenant/list?limit=100000` and the `total_count` field before suspecting data loss. |
| `console subscriber server failed: Address already in use` panic at startup | **Benign** for co-located multi-node drills — all three nodes bind the same tokio-console port. Does not affect Raft or shutdown. Do NOT count it as a Raft panic; grep `invalid state: expect` to isolate the real Raft panic. |
| Process won't exit on `kill -INT`, only `kill -9` works | Regression in signal handling (libc handler in daemon.rs) or a `leave_cluster` re-introduced into shutdown. |
