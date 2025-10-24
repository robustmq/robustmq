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

# Parse arguments
START_BROKER=false
if [ "$1" == "--start-broker" ]; then
    START_BROKER=true
fi

# Cleanup function
cleanup() {
    if [ "$START_BROKER" == "true" ] && [ ! -z "$BROKER_PID" ]; then
        echo "Stopping broker-server..."
        kill $BROKER_PID 2>/dev/null || true
    fi
}

# Register cleanup on exit
trap cleanup EXIT

# Start broker if needed
if [ "$START_BROKER" == "true" ]; then
    echo "Starting broker-server..."
    cargo run --package cmd --bin broker-server >> 1.log 2>&1 &
    BROKER_PID=$!
    echo "Waiting for broker to be ready..."
    sleep 30
else
    echo "Skipping broker startup (assuming broker is already running)..."
fi

# Run tests
echo "Running integration tests..."
cargo nextest run --fail-fast \
  --package grpc-clients \
  --package robustmq-test
