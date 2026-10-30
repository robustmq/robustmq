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

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$ROOT_DIR/data/cluster-logs"
CLUSTER_START_TIMEOUT="${CLUSTER_START_TIMEOUT:-120}"

wait_for_cluster_members() {
    local expected_ids="$1"
    local description="$2"
    local deadline=$((SECONDS + CLUSTER_START_TIMEOUT))

    echo "Waiting for $description..."
    while (( SECONDS < deadline )); do
        local response
        response=$(curl -fsS --max-time 2 "http://127.0.0.1:58080/api/info" 2>/dev/null) || {
            sleep 1
            continue
        }

        if printf '%s' "$response" | python3 -c '
import json
import sys

try:
    payload = json.load(sys.stdin)
    data = payload.get("data") or {}
    meta = data.get("meta") or {}
    status = meta.get("metadata_0") or {}
    membership = status.get("membership_config", {}).get("membership", {})
    voters = {
        int(node_id)
        for config in membership.get("configs", [])
        for node_id in config
    }
    brokers = {
        int(node["node_id"])
        for node in data.get("broker_node_list", [])
        if "node_id" in node
    }
    expected = {int(node_id) for node_id in sys.argv[1].split(",")}
    ready = (
        payload.get("code") == 0
        and status.get("current_leader") is not None
        and expected.issubset(voters)
        and expected.issubset(brokers)
    )
    sys.exit(0 if ready else 1)
except (KeyError, TypeError, ValueError, json.JSONDecodeError):
    sys.exit(1)
' "$expected_ids"; then
            echo "Cluster state is ready: $description"
            return 0
        fi

        sleep 1
    done

    echo "Timed out waiting for $description after ${CLUSTER_START_TIMEOUT}s"
    return 1
}

show_startup_logs() {
    echo "Cluster startup failed. Logs in $LOG_DIR:"
    for i in 1 2 3; do
        local log="$LOG_DIR/server-${i}.log"
        echo "--- server-${i} log (last 80 lines) ---"
        tail -n 80 "$log" 2>/dev/null || true
    done
}

start() {
    mkdir -p "$LOG_DIR"

    echo "Building broker-server..."
    cargo_args=(
        build
        --package cmd
        --bin broker-server
        --bin cli-command
        --manifest-path "$ROOT_DIR/Cargo.toml"
    )
    if [[ -n "${BROKER_FEATURES:-}" ]]; then
        echo "Enabling broker features: $BROKER_FEATURES"
        cargo_args+=(--features "$BROKER_FEATURES")
    fi
    cargo "${cargo_args[@]}"

    for i in 1 2 3; do
        local conf="$ROOT_DIR/config/cluster/server-${i}.toml"
        local log="$LOG_DIR/server-${i}.log"

        if pgrep -f "server-${i}.toml" > /dev/null 2>&1; then
            echo "server-${i} is already running"
            continue
        fi

        echo "Starting server-${i}..."
        (cd "$ROOT_DIR" && "$ROOT_DIR/target/debug/broker-server" --conf "$conf") > "$log" 2>&1 &
        echo "server-${i} started (log: $log)"

        case "$i" in
            1) expected_ids="1" ;;
            2) expected_ids="1,2" ;;
            3) expected_ids="1,2,3" ;;
        esac
        if ! wait_for_cluster_members "$expected_ids" "metadata voters and registered brokers [$expected_ids]"; then
            show_startup_logs
            exit 1
        fi
    done

    echo ""
    echo "All three brokers are registered and present in Raft membership."

    for i in $(seq 1 30); do
        if (cd "$ROOT_DIR" && "$ROOT_DIR/target/debug/cli-command" cluster status) 2>/dev/null; then
            echo ""
            echo "Cluster is up!"
            exit 0
        fi
        echo "  Waiting... (${i}/30)"
        sleep 2
    done

    echo "Cluster CLI did not become ready in time."
    show_startup_logs
    exit 1
}

stop() {
    local pids
    pids=$(pgrep -f "config/cluster/server-[123].toml" 2>/dev/null || true)

    if [[ -z "$pids" ]]; then
        echo "No cluster nodes are running."
        return
    fi

    echo "Stopping cluster nodes (pids: $pids)..."
    echo "$pids" | xargs kill
    echo "All nodes stopped."
}

case "${1:-}" in
    start) start ;;
    stop)  stop  ;;
    *)
        echo "Usage: $0 {start|stop}"
        exit 1
        ;;
esac
