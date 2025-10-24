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
    echo "Starting broker-server (compiling and launching)..."
    cargo run --package cmd --bin broker-server >> 1.log 2>&1 &
    BROKER_PID=$!
    
    echo "Waiting for broker to be ready..."
    MAX_WAIT=1800  # Maximum wait time in seconds (30 minutes for compilation + startup)
    RETRY_INTERVAL=3
    BROKER_READY=false
    
    for ((ELAPSED=0; ELAPSED<MAX_WAIT; ELAPSED+=RETRY_INTERVAL)); do
        # Check if process is still running
        if ! kill -0 $BROKER_PID 2>/dev/null; then
            echo "❌ Broker process died unexpectedly"
            echo "Last 50 lines of broker log:"
            tail -n 50 1.log
            exit 1
        fi
        
        # Check MQTT port 1883 (primary service port)
        if nc -z 127.0.0.1 1883 2>/dev/null || \
           (command -v lsof >/dev/null 2>&1 && lsof -i:1883 -sTCP:LISTEN >/dev/null 2>&1); then
            echo "✅ Broker is ready after ${ELAPSED}s (MQTT port 1883 is listening)"
            BROKER_READY=true
            break
        fi
        
        echo "⏳ Waiting for broker... (${ELAPSED}s/${MAX_WAIT}s)"
        sleep $RETRY_INTERVAL
    done
    
    if [ "$BROKER_READY" != "true" ]; then
        echo "❌ Broker failed to start within ${MAX_WAIT}s"
        echo "Last 50 lines of broker log:"
        tail -n 50 1.log
        exit 1
    fi
    
    # Give it a few more seconds to stabilize
    echo "Waiting 5s for broker to stabilize..."
    sleep 5
else
    echo "Skipping broker startup (assuming broker is already running)..."
fi

# Run tests
echo "Running integration tests..."
cargo nextest run --fail-fast \
  --package grpc-clients \
  --package robustmq-test
