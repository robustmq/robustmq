#!/bin/bash
# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -u

TEST_FILTER="${1:-}"
if [ -z "$TEST_FILTER" ]; then
    echo "Usage: $0 <test-filter>"
    exit 1
fi

DRILL_ATTEMPTS="${DRILL_ATTEMPTS:-2}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

BIN=./target/debug/broker-server

# ── stop any node and wipe state (graceful SIGINT; SIGKILL can leave a stuck socket) ──
stop_cluster() {
    for c in 1 2 3; do
        pid=$(pgrep -f "server-${c}.toml" || true)
        [ -n "$pid" ] && kill -INT $pid 2>/dev/null || true
    done
    for c in 1 2 3; do
        t0=$(date +%s)
        while pgrep -f "server-${c}.toml" >/dev/null 2>&1; do
            sleep 1
            [ $(($(date +%s) - t0)) -gt 30 ] && { kill -9 "$(pgrep -f "server-${c}.toml")" 2>/dev/null || true; break; }
        done
    done
}

clean_state() {
    stop_cluster
    rm -rf data/broker-1 data/broker-2 data/broker-3 data/logs
    rm -f /tmp/n1.log /tmp/n2.log /tmp/n3.log
}

cleanup() {
    echo "Cleaning up cluster..."
    stop_cluster
}
trap cleanup EXIT

# Start a fresh 3-node cluster and wait until membership is [1,2,3]. Returns non-zero
# if the cluster never forms.
start_cluster() {
    clean_state
    # staggered so node1 bootstraps the cluster before peers join
    echo "Starting 3-node cluster (logs: /tmp/n{1,2,3}.log)..."
    "$BIN" --conf config/cluster/server-1.toml > /tmp/n1.log 2>&1 &
    sleep 8
    "$BIN" --conf config/cluster/server-2.toml > /tmp/n2.log 2>&1 &
    sleep 9
    "$BIN" --conf config/cluster/server-3.toml > /tmp/n3.log 2>&1 &
    sleep 13

    echo "Waiting for cluster membership [1,2,3]..."
    for i in $(seq 1 60); do
        NODES=$(curl -s http://127.0.0.1:58080/api/info 2>/dev/null \
            | python3 -c "import sys,json;d=json.load(sys.stdin).get('data',{});print(sorted(n.get('node_id') for n in d.get('broker_node_list',[])))" 2>/dev/null || true)
        if [ "$NODES" = "[1, 2, 3]" ]; then
            echo "Cluster ready: nodes $NODES (after $((i * 2))s)"
            sleep 5  # let replication/leases settle
            return 0
        fi
        sleep 2
    done
    echo "❌ Cluster did not form membership [1,2,3] in time"
    echo "--- /tmp/n1.log tail ---"; tail -30 /tmp/n1.log 2>/dev/null || true
    return 1
}

echo "=========================================="
echo "Chaos drill: $TEST_FILTER (max attempts: $DRILL_ATTEMPTS)"
echo "=========================================="

echo "Building broker-server..."
cargo build --package cmd --bin broker-server

RESULT=1
for attempt in $(seq 1 "$DRILL_ATTEMPTS"); do
    echo ""
    echo "########## attempt $attempt/$DRILL_ATTEMPTS: $TEST_FILTER ##########"

    if ! start_cluster; then
        echo "attempt $attempt: cluster failed to start"
        RESULT=1
        continue
    fi

    # Use `--test mod` so the drill runs exactly once: drill files are picked up both as a
    # standalone test binary AND via tests/tests/mod.rs, so without this it runs twice
    # (the second run inheriting an already-churned cluster and flaking).
    echo "Running drill: cargo test -p robustmq-test --test mod $TEST_FILTER -- --ignored --nocapture"
    cargo test -p robustmq-test --test mod "$TEST_FILTER" -- --ignored --nocapture
    RESULT=$?

    if [ $RESULT -eq 0 ]; then
        echo "✅ Drill '$TEST_FILTER' passed on attempt $attempt"
        break
    fi
    echo "❌ Drill '$TEST_FILTER' failed on attempt $attempt (exit $RESULT)"
    if [ "$attempt" -lt "$DRILL_ATTEMPTS" ]; then
        echo "Retrying on a fresh cluster..."
        stop_cluster
    fi
done

echo "=========================================="
if [ $RESULT -eq 0 ]; then
    echo "✅ Drill '$TEST_FILTER' PASSED"
else
    echo "❌ Drill '$TEST_FILTER' FAILED after $DRILL_ATTEMPTS attempt(s)"
fi
exit $RESULT
