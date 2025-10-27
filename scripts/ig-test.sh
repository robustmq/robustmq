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
    REQUIRED_PORTS=(1228 8080 9091 6777 1883 1885 8083 8085 9083)
    
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
        
        # Show existing broker processes before killing
        if pgrep -l broker-server >/dev/null 2>&1; then
            echo "Found broker-server processes:"
            pgrep -la broker-server || true
        else
            echo "No broker-server processes found"
        fi
        
        # Kill with extreme prejudice
        pkill -9 broker-server 2>/dev/null || true
        killall -9 broker-server 2>/dev/null || true
        
        # Also try to kill processes on specific ports using fuser
        echo "Attempting to kill processes on occupied ports..."
        for port in "${PORTS_IN_USE[@]}"; do
            if command -v fuser >/dev/null 2>&1; then
                fuser -k -9 $port/tcp 2>/dev/null || true
            fi
            
            # Alternative: use lsof to find and kill
            if command -v lsof >/dev/null 2>&1; then
                PIDS=$(lsof -ti:$port 2>/dev/null || true)
                if [ ! -z "$PIDS" ]; then
                    echo "  Killing PIDs on port $port: $PIDS"
                    kill -9 $PIDS 2>/dev/null || true
                fi
            fi
        done
        
        echo "Sent kill signal to all broker-server processes and port occupants"
        sleep 2
        
        # Verify processes are gone
        if pgrep broker-server >/dev/null 2>&1; then
            echo "⚠️  Warning: Some broker-server processes still exist after kill"
            pgrep -la broker-server || true
        else
            echo "✅ All broker-server processes terminated"
        fi
        
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
                # Show port status every 10 seconds
                if [ $((CLEANUP_ELAPSED % 10)) -eq 0 ] && command -v ss >/dev/null 2>&1; then
                    echo "⏳ Still waiting... ${#STILL_IN_USE[@]} port(s) still in use: ${STILL_IN_USE[@]} (${CLEANUP_ELAPSED}s/${MAX_WAIT_CLEANUP}s)"
                    echo "   Port states:"
                    for stuck_port in "${STILL_IN_USE[@]}"; do
                        PORT_STATE=$(ss -tan state all "sport = :$stuck_port" 2>/dev/null | grep -v "State" | head -1 || echo "Unknown")
                        if echo "$PORT_STATE" | grep -q "TIME-WAIT"; then
                            echo "   - Port $stuck_port: TIME-WAIT (TCP cleanup, will expire soon)"
                        elif echo "$PORT_STATE" | grep -q "LISTEN"; then
                            echo "   - Port $stuck_port: LISTEN (active process)"
                        elif [ -n "$PORT_STATE" ]; then
                            echo "   - Port $stuck_port: $PORT_STATE"
                        else
                            echo "   - Port $stuck_port: Unknown state"
                        fi
                    done
                else
                    echo "⏳ Still waiting... ${#STILL_IN_USE[@]} port(s) still in use: ${STILL_IN_USE[@]} (${CLEANUP_ELAPSED}s/${MAX_WAIT_CLEANUP}s)"
                fi
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
            echo "⚠️  WARNING: Some ports still show as occupied after ${MAX_WAIT_CLEANUP}s"
            echo "Ports: ${FINAL_CHECK[@]}"
            echo ""
            
            # Check if ports are in TIME_WAIT state or have identifiable processes
            TIME_WAIT_PORTS=()
            GHOST_PORTS=()
            ACTIVE_PORTS=()
            
            for port in "${FINAL_CHECK[@]}"; do
                # Check port state
                IS_TIME_WAIT=false
                IS_LISTEN=false
                HAS_PROCESS=false
                
                if command -v ss >/dev/null 2>&1; then
                    if ss -tan | grep ":$port " | grep -q "TIME-WAIT"; then
                        IS_TIME_WAIT=true
                    elif ss -tan | grep ":$port " | grep -q "LISTEN"; then
                        IS_LISTEN=true
                    fi
                fi
                
                # Check if we can find the process
                if command -v lsof >/dev/null 2>&1; then
                    if lsof -i:$port 2>/dev/null | grep -q LISTEN; then
                        HAS_PROCESS=true
                    fi
                fi
                
                # Categorize the port
                if [ "$IS_TIME_WAIT" = true ]; then
                    TIME_WAIT_PORTS+=($port)
                elif [ "$IS_LISTEN" = true ] && [ "$HAS_PROCESS" = false ]; then
                    GHOST_PORTS+=($port)  # LISTEN but no identifiable process
                elif [ "$IS_LISTEN" = true ] && [ "$HAS_PROCESS" = true ]; then
                    ACTIVE_PORTS+=($port)  # Real active process
                else
                    TIME_WAIT_PORTS+=($port)  # Unknown, assume safe
                fi
            done
            
            if [ ${#ACTIVE_PORTS[@]} -eq 0 ]; then
                echo "✅ Good news: No active processes blocking the ports"
                if [ ${#TIME_WAIT_PORTS[@]} -gt 0 ]; then
                    echo "   TIME-WAIT ports (harmless): ${TIME_WAIT_PORTS[@]}"
                fi
                if [ ${#GHOST_PORTS[@]} -gt 0 ]; then
                    echo "   Ghost ports (no process found, likely safe): ${GHOST_PORTS[@]}"
                    echo "   ℹ️  These ports show as LISTEN but no process was found"
                    echo "   ℹ️  Could be zombie processes or permission issues"
                    echo "   ℹ️  Broker will attempt to bind anyway with SO_REUSEADDR"
                fi
                echo "   Continuing with broker startup..."
                echo "=========================================="
                echo ""
            else
                echo "❌ FAILED: Some ports have identifiable active processes"
                echo "Active ports with processes: ${ACTIVE_PORTS[@]}"
                if [ ${#GHOST_PORTS[@]} -gt 0 ]; then
                    echo "Ghost ports (no process): ${GHOST_PORTS[@]}"
                fi
                if [ ${#TIME_WAIT_PORTS[@]} -gt 0 ]; then
                    echo "TIME-WAIT ports (harmless): ${TIME_WAIT_PORTS[@]}"
                fi
                echo ""
                echo "Detailed investigation:"
                for port in "${ACTIVE_PORTS[@]}"; do
                    echo ""
                    echo "Port $port:"
                    get_port_info $port
                done
                echo ""
                echo "=========================================="
                echo "Manual intervention required."
                echo "Possible causes:"
                echo "  - Other services using these ports"
                echo "  - Docker containers"
                echo "  - System services"
                echo ""
                echo "Manual cleanup commands:"
                echo "  - Check processes: sudo lsof -i:<PORT>"
                echo "  - Check connections: ss -tlnp | grep <PORT>"
                echo "  - Check Docker: docker ps"
                echo "=========================================="
                exit 1
            fi
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
