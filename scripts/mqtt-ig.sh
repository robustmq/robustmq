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

VERSION=`grep '^version = ' Cargo.toml | head -n1 | cut -d'"' -f2`
echo "Version: ${VERSION}"

# Clean test artifacts to save disk space (preserve build cache)
echo "Cleaning test artifacts..."
rm -rf target/nextest
rm -rf target/debug/incremental
find target -type f -name "*-????????????????" -delete 2>/dev/null || true

# Build broker-server in release mode (saves ~60% disk space vs debug)
echo "Building broker-server in release mode..."
cargo build --release --package cmd --bin broker-server

# Strip debug symbols to save more space (~30% reduction)
strip target/release/broker-server 2>/dev/null || true

# Start broker in background
echo "Starting broker-server..."
nohup target/release/broker-server >> 1.log 2>&1 &

# Wait for broker to start
sleep 5

# Run integration tests
echo "Running integration tests..."
echo "1/4 Running grpc-clients meta tests..."
cargo nextest run --package grpc-clients --test mod -- meta

echo "2/4 Running robustmq-test meta tests..."
cargo nextest run --package robustmq-test --test mod -- meta

echo "3/4 Running journal tests..."
cargo nextest run --package robustmq-test --test mod -- journal

echo "4/4 Running MQTT tests..."
cargo nextest run --package robustmq-test --test mod -- mqtt

# Clean up broker artifacts after tests
echo "Cleaning broker artifacts..."
rm -rf target/release/broker-server
rm -rf target/release/deps/broker_server*
rm -rf target/release/incremental
