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
    PORT=$1
    if nc -z 127.0.0.1 $PORT 2>/dev/null || \
       (command -v lsof >/dev/null 2>&1 && lsof -i:$PORT -sTCP:LISTEN >/dev/null 2>&1); then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Function to get detailed port information
get_port_info() {
    PORT=$1
    echo "  Checking port $PORT with multiple tools..."
    
    # Method 1: lsof
    if command -v lsof >/dev/null 2>&1; then
        echo "  [lsof]"
        lsof -i:$PORT 2>/dev/null || echo "    No results from lsof"
    fi
    
    # Method 2: netstat
    if command -v netstat >/dev/null 2>&1; then
        echo "  [netstat]"
        netstat -tlnp 2>/dev/null | grep ":$PORT " || echo "    No results from netstat"
    fi
    
    # Method 3: ss (modern alternative)
    if command -v ss >/dev/null 2>&1; then
        echo "  [ss]"
        ss -tlnp 2>/dev/null | grep ":$PORT " || echo "    No results from ss"
    fi
    
    # Method 4: fuser
    if command -v fuser >/dev/null 2>&1; then
        echo "  [fuser]"
        fuser $PORT/tcp 2>/dev/null || echo "    No results from fuser"
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
    
    # Function to check all ports and return the list of occupied ones
    check_all_ports() {
        CHECK_OCCUPIED=()
        for port in "${REQUIRED_PORTS[@]}"; do
            if check_port $port; then
                CHECK_OCCUPIED+=($port)
            fi
        done
        echo "${CHECK_OCCUPIED[@]}"
    }
    
    # Initial port check
    PORTS_IN_USE=($(check_all_ports))
    
    # Display initial port status
    for port in "${REQUIRED_PORTS[@]}"; do
        if check_port $port; then
            echo "❌ Port $port is already in use"
        else
            echo "✅ Port $port is available"
        fi
    done
    
    # If any port is in use, try aggressive cleanup
    if [ ${#PORTS_IN_USE[@]} -gt 0 ]; then
        echo ""
        echo "=========================================="
        echo "🔧 AUTOMATIC CLEANUP: Killing old processes"
        echo "=========================================="
        echo "${#PORTS_IN_USE[@]} port(s) occupied: ${PORTS_IN_USE[@]}"
        echo ""
        
        # Step 1: Kill all broker-server processes aggressively
        echo "Step 1: Killing all broker-server processes..."
        pkill -9 broker-server 2>/dev/null || true
        echo "Sent kill signal to all broker-server processes"
        
        # Step 2: Wait for ports to be released with retry mechanism
        echo ""
        echo "Step 2: Waiting for ports to be released..."
        MAX_WAIT_CLEANUP=60  # Maximum 60 seconds to wait for cleanup
        RETRY_INTERVAL=2
        CLEANUP_ELAPSED=0
        
        while [ $CLEANUP_ELAPSED -lt $MAX_WAIT_CLEANUP ]; do
            sleep $RETRY_INTERVAL
            CLEANUP_ELAPSED=$((CLEANUP_ELAPSED + RETRY_INTERVAL))
            
            # Re-check all ports
            STILL_IN_USE=($(check_all_ports))
            
            if [ ${#STILL_IN_USE[@]} -eq 0 ]; then
                echo "✅ All ports released after ${CLEANUP_ELAPSED}s"
                break
            else
                echo "⏳ Still waiting... ${#STILL_IN_USE[@]} port(s) still in use: ${STILL_IN_USE[@]} (${CLEANUP_ELAPSED}s/${MAX_WAIT_CLEANUP}s)"
            fi
        done
        
        # Step 3: Final verification
        echo ""
        echo "Step 3: Final port verification..."
        FINAL_CHECK=($(check_all_ports))
        
        if [ ${#FINAL_CHECK[@]} -eq 0 ]; then
            echo "✅ SUCCESS: All ports are now available"
            echo "Continuing with broker startup..."
            echo "=========================================="
            echo ""
        else
            echo "❌ FAILED: Some ports are still occupied after ${MAX_WAIT_CLEANUP}s"
            echo "Ports still in use: ${FINAL_CHECK[@]}"
            echo ""
            echo "Detailed investigation:"
            for port in "${FINAL_CHECK[@]}"; do
                echo ""
                echo "Port $port:"
                get_port_info $port
            done
            echo ""
            echo "=========================================="
            echo "Manual intervention required."
            echo "Possible causes:"
            echo "  - TCP TIME_WAIT state (wait 1-2 minutes)"
            echo "  - Other services using these ports"
            echo "  - Zombie/orphan processes"
            echo ""
            echo "Manual cleanup commands:"
            echo "  - Check processes: lsof -i:<PORT>"
            echo "  - Check connections: netstat -an | grep <PORT>"
            echo "  - Force kill: pkill -9 broker-server"
            echo "=========================================="
            exit 1
        fi
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
