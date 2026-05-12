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

start() {
    mkdir -p "$LOG_DIR"

    echo "Building broker-server..."
    cargo build --package cmd --bin broker-server --manifest-path "$ROOT_DIR/Cargo.toml"

    for i in 1 2 3; do
        local conf="$ROOT_DIR/config/cluster/server-${i}.toml"
        local log="$LOG_DIR/server-${i}.log"

        if pgrep -f "server-${i}.toml" > /dev/null 2>&1; then
            echo "server-${i} is already running"
            continue
        fi

        echo "Starting server-${i}..."
        cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --package cmd --bin broker-server -- --conf "$conf" > "$log" 2>&1 &
        echo "server-${i} started (log: $log)"
    done

    echo ""
    echo "Waiting for cluster to be ready..."
    sleep 5

    for i in $(seq 1 30); do
        if cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --package cmd --bin cli-command -- cluster status 2>/dev/null; then
            echo ""
            echo "Cluster is up!"
            exit 0
        fi
        echo "  Waiting... (${i}/30)"
        sleep 2
    done

    echo "Cluster did not become ready in time. Check logs in $LOG_DIR"
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
