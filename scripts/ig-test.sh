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
        # Stop the 3-node cluster started for the test run.
        echo "Stopping cluster..."
        /bin/bash ./scripts/cluster.sh stop 2>/dev/null || true
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
    REQUIRED_PORTS=(1228 58080 9091 6777 1883 1885 8083 8085 9083)

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
        echo "🔧 Auto cleanup (${#PORTS_IN_USE[@]} port(s): ${PORTS_IN_USE[@]})"
        echo "=========================================="

        # Kill broker-server processes
        if pgrep broker-server >/dev/null 2>&1; then
            echo "Step 1: Terminating broker-server processes..."
            pkill -9 broker-server 2>/dev/null || true
            killall -9 broker-server 2>/dev/null || true
        else
            echo "Step 1: No broker-server processes"
        fi

        # Kill processes on occupied ports
        for port in "${PORTS_IN_USE[@]}"; do
            if command -v fuser >/dev/null 2>&1; then
                fuser -k -9 $port/tcp 2>/dev/null || true
            fi
            if command -v lsof >/dev/null 2>&1; then
                lsof -ti:$port 2>/dev/null | xargs -r kill -9 2>/dev/null || true
            fi
        done

        sleep 2

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
                # Only show progress every 5 seconds to reduce noise
                if [ $((CLEANUP_ELAPSED % 5)) -eq 0 ]; then
                    echo "⏳ Waiting... ${#STILL_IN_USE[@]} port(s) occupied: ${STILL_IN_USE[@]} (${CLEANUP_ELAPSED}s/${MAX_WAIT_CLEANUP}s)"
                fi
            fi
        done

        # Step 3: Final verification - STRICT mode
        echo ""
        echo "Step 3: Final port verification (STRICT)..."
        FINAL_CHECK=($(check_all_ports))

        if [ ${#FINAL_CHECK[@]} -eq 0 ]; then
            echo "✅ SUCCESS: All ports are now available"
            echo "Continuing with broker startup..."
            echo "=========================================="
            echo ""
        else
            # STRICT: Any port still occupied = FAIL
            echo "❌ FAILED: Ports still occupied after ${MAX_WAIT_CLEANUP}s cleanup attempt"
            echo "Occupied ports: ${FINAL_CHECK[@]}"
            echo ""
            echo "Detailed diagnostics:"
            for port in "${FINAL_CHECK[@]}"; do
                echo ""
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                echo "Port $port:"
                get_port_info $port
            done
            echo ""
            echo "=========================================="
            echo "❌ Port cleanup failed"
            echo ""
            echo "Next steps:"
            echo "  1. Wait 1-2 minutes for TCP TIME_WAIT to expire"
            echo "  2. Check for other services: sudo lsof -i:<PORT>"
            echo "  3. Check Docker containers: docker ps"
            echo "  4. Manually kill processes: sudo fuser -k <PORT>/tcp"
            echo "  5. Reboot if necessary"
            echo "=========================================="
            exit 1
        fi
    fi

    echo "✅ All required ports are available"
    echo ""

    echo "Starting 3-node cluster via scripts/cluster.sh..."
    echo "=========================================="

    # Start a 3-node cluster (builds broker-server, launches server-1/2/3, waits until
    # the cluster reports ready). Integration tests run against this cluster so multi-node
    # paths (replication / ISR / leader routing) are exercised, not just a single node.
    if ! /bin/bash ./scripts/cluster.sh start; then
        echo ""
        echo "=========================================="
        echo "❌ Cluster failed to start"
        echo "=========================================="
        exit 1
    fi

    # Give it a few more seconds to stabilize
    echo "Waiting 5s for cluster to stabilize..."
    sleep 5
else
    echo "Skipping broker startup (assuming broker is already running)..."
fi

# Run tests
echo "Running integration tests..."
# Integration tests all hit a single shared broker, so they are broker-bound, not
# CPU-parallel. nextest's default profile uses 14 test threads; on a 4-core CI runner
# that oversubscribes the CPU ~3.5x and starves the broker, causing request timeouts
# (e.g. POST /mcp blocking 10s -> mcp_test failures). Scale concurrency to the available
# cores so tests and the broker are not fighting over the CPU.
TEST_THREADS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
echo "Using --test-threads=${TEST_THREADS} (detected CPU cores)"
cargo nextest run --fail-fast \
  --test-threads="${TEST_THREADS}" \
  --package grpc-clients \
  --package robustmq-test

# Kafka Java-client integration tests hit the same running broker.
echo "Running Kafka Java-client integration tests..."
if ! command -v mvn >/dev/null 2>&1; then
    echo "ERROR: mvn (Maven) not found; it is required for the Kafka Java integration tests."
    exit 1
fi
(cd tests/kafka-java && mvn -q test)
