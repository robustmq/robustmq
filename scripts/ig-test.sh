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
    if [ "$START_BROKER" == "true" ]; then
        # Stop tail process if running
        if [ ! -z "$TAIL_PID" ]; then
            kill $TAIL_PID 2>/dev/null || true
        fi
        # Stop broker if running
        if [ ! -z "$BROKER_PID" ]; then
            echo "Stopping broker-server..."
            kill $BROKER_PID 2>/dev/null || true
        fi
    fi
}

# Function to check if port is in use
check_port() {
    local port=$1
    if nc -z 127.0.0.1 $port 2>/dev/null || \
       (command -v lsof >/dev/null 2>&1 && lsof -i:$port -sTCP:LISTEN >/dev/null 2>&1); then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Register cleanup on exit
trap cleanup EXIT

# Start broker if needed
if [ "$START_BROKER" == "true" ]; then
    echo "Checking if required ports are available..."
    echo "=========================================="
    
    # List of ports that broker-server needs
    REQUIRED_PORTS=(1228 8080 9091 6777 1883 1884 8083 8084 9083)
    PORTS_IN_USE=()
    
    # Check each port
    for port in "${REQUIRED_PORTS[@]}"; do
        if check_port $port; then
            PORTS_IN_USE+=($port)
            echo "❌ Port $port is already in use"
        fi
    done
    
    # If any port is in use, report and exit
    if [ ${#PORTS_IN_USE[@]} -gt 0 ]; then
        echo ""
        echo "=========================================="
        echo "❌ ERROR: Cannot start broker-server"
        echo "=========================================="
        echo "The following ports are already in use:"
        for port in "${PORTS_IN_USE[@]}"; do
            echo "  - Port $port"
            # Show which process is using the port
            if command -v lsof >/dev/null 2>&1; then
                echo "    Process: $(lsof -i:$port -sTCP:LISTEN -t 2>/dev/null | head -1 | xargs ps -p 2>/dev/null | tail -1 || echo 'Unknown')"
            fi
        done
        echo ""
        echo "Please stop the conflicting processes and try again."
        echo "=========================================="
        exit 1
    fi
    
    echo "✅ All required ports are available"
    echo ""
    
    echo "Building broker-server..."
    echo "=========================================="
    
    # Build broker-server (this will use cache effectively)
    cargo build --package cmd --bin broker-server
    
    echo ""
    echo "Starting broker-server..."
    echo "Broker logs will be written to 1.log and tailed below..."
    echo "=========================================="
    
    # Start broker and redirect output to file
    ./target/debug/broker-server > 1.log 2>&1 &
    BROKER_PID=$!
    
    # Start tailing the log in background
    tail -f 1.log &
    TAIL_PID=$!
    
    echo "Waiting for broker to be ready..."
    MAX_WAIT=1800  # Maximum wait time in seconds (30 minutes for compilation + startup)
    RETRY_INTERVAL=3
    BROKER_READY=false
    
    for ((ELAPSED=0; ELAPSED<MAX_WAIT; ELAPSED+=RETRY_INTERVAL)); do
        # Check if process is still running
        if ! kill -0 $BROKER_PID 2>/dev/null; then
            echo ""
            echo "=========================================="
            echo "❌ Broker process died unexpectedly"
            echo "=========================================="
            kill $TAIL_PID 2>/dev/null || true
            exit 1
        fi
        
        # Check MQTT port 1883 (primary service port)
        if nc -z 127.0.0.1 1883 2>/dev/null || \
           (command -v lsof >/dev/null 2>&1 && lsof -i:1883 -sTCP:LISTEN >/dev/null 2>&1); then
            echo ""
            echo "=========================================="
            echo "✅ Broker is ready after ${ELAPSED}s (MQTT port 1883 is listening)"
            echo "=========================================="
            # Stop tailing the log
            kill $TAIL_PID 2>/dev/null || true
            BROKER_READY=true
            break
        fi
        
        sleep $RETRY_INTERVAL
    done
    
    if [ "$BROKER_READY" != "true" ]; then
        echo ""
        echo "=========================================="
        echo "❌ Broker failed to start within ${MAX_WAIT}s"
        echo "=========================================="
        kill $TAIL_PID 2>/dev/null || true
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
