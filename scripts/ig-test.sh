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


# Exit on error
set -e

# Build broker-server binary
echo "Building broker-server binary..."
cargo build --release --package cmd --bin broker-server

# Build test binaries
echo "Building test binaries..."
cargo build --release \
  --package grpc-clients \
  --package robustmq-test \
  --tests

# Start broker
echo "Starting broker-server..."
strip target/release/broker-server 2>/dev/null || true
nohup target/release/broker-server >> 1.log 2>&1 &
BROKER_PID=$!
sleep 30

# Run tests (no compilation needed, binaries already built)
echo "Running integration tests..."
# Temporarily disable exit on error to ensure cleanup happens
set +e
cargo nextest run --release --fail-fast \
  --package grpc-clients \
  --package robustmq-test

TEST_EXIT_CODE=$?
set -e

# Stop broker
echo "Stopping broker-server..."
kill $BROKER_PID 2>/dev/null || true

# Exit with the test exit code
if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo "Tests failed with exit code $TEST_EXIT_CODE"
    exit $TEST_EXIT_CODE
fi

echo "All tests passed successfully"
