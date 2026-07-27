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

SUITE=""
START_BROKER=false
for arg in "$@"; do
    case "$arg" in
        --start-broker) START_BROKER=true ;;
        core|mqtt|nats|mq9|kafka|amqp) SUITE="$arg" ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 <core|mqtt|nats|mq9|kafka|amqp> [--start-broker]"
            exit 1
            ;;
    esac
done

if [ -z "$SUITE" ]; then
    echo "Usage: $0 <core|mqtt|nats|mq9|kafka|amqp> [--start-broker]"
    exit 1
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

# Integration tests all hit a single shared broker, so they are broker-bound, not
# CPU-parallel. nextest's default profile uses 14 test threads; on a 4-core CI runner
# that oversubscribes the CPU ~3.5x and starves the broker, causing request timeouts
# (e.g. POST /mcp blocking 10s -> mcp_test failures). Scale concurrency to the available
# cores so tests and the broker are not fighting over the CPU.
TEST_THREADS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

# tests/tests/mod.rs compiles amqp/engine/kafka/mq9/mqtt/nats as submodules of one
# "mod" nextest binary; --test mod -E 'test(/^<mod>::/)' isolates just that
# protocol's tests within it without touching the others.
run_mod_submodule() {
    local module="$1"
    echo "Using --test-threads=${TEST_THREADS} (detected CPU cores)"
    cargo nextest run --fail-fast \
      --test-threads="${TEST_THREADS}" \
      --package robustmq-test \
      --test mod \
      -E "test(/^${module}::/)"
}

require_mvn() {
    if ! command -v mvn >/dev/null 2>&1; then
        echo "ERROR: mvn (Maven) not found; it is required for the $1 Java integration tests."
        exit 1
    fi
}

case "$SUITE" in
    core)
        echo "Running core (protocol-agnostic) integration tests..."
        echo "Using --test-threads=${TEST_THREADS} (detected CPU cores)"
        # -E applies across the WHOLE selection, not just one --test binary, so
        # each binary this suite owns must be spelled out (or, for "mod",
        # explicitly restricted to its engine:: submodule) rather than
        # filtering on test name alone -- that would also silently drop the
        # config_test/group_gc/etc. binaries, which don't have "engine::"
        # names of their own.
        cargo nextest run --fail-fast \
          --test-threads="${TEST_THREADS}" \
          --package grpc-clients \
          --package robustmq-test \
          -E 'package(grpc-clients)
              or binary_id(robustmq-test::config_test)
              or binary_id(robustmq-test::group_gc)
              or binary_id(robustmq-test::mcp_test)
              or binary_id(robustmq-test::node_call)
              or binary_id(robustmq-test::offset_test)
              or binary_id(robustmq-test::topic_test)
              or (binary_id(robustmq-test::mod) and test(/^engine::/))'
        ;;
    mqtt)
        echo "Running MQTT integration tests..."
        echo "Using --test-threads=${TEST_THREADS} (detected CPU cores)"
        # MQTT tests live in two places: submodule mqtt:: inside the "mod"
        # binary, and a couple of inline unit tests in the package's own lib
        # target -- both need --lib alongside --test mod to be picked up.
        cargo nextest run --fail-fast \
          --test-threads="${TEST_THREADS}" \
          --package robustmq-test \
          --lib \
          --test mod \
          -E 'test(/^mqtt::/)'
        ;;
    nats)
        echo "Running NATS integration tests..."
        run_mod_submodule nats
        ;;
    mq9)
        echo "Running MQ9 integration tests..."
        run_mod_submodule mq9
        ;;
    kafka)
        echo "Running Kafka integration tests..."
        run_mod_submodule kafka

        echo "Running Kafka Java-client integration tests..."
        require_mvn Kafka
        (cd tests/kafka-java && mvn -q test)
        ;;
    amqp)
        echo "Running AMQP integration tests..."
        run_mod_submodule amqp

        echo "Running RabbitMQ Java-client integration tests..."
        require_mvn RabbitMQ
        (cd tests/rabbitmq-java && mvn -q test)
        ;;
esac
