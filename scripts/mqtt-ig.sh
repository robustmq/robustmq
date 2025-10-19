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

# Skip cargo clean in CI to preserve cached dependencies
# Only clean locally to ensure fresh builds
if [ -z "$CI" ]; then
  echo "Running cargo clean (local mode)"
  cargo clean
else
  echo "Skipping cargo clean (CI mode - using cached builds)"
  # Clean only test artifacts to save space
  rm -rf target/nextest
  rm -rf target/debug/incremental
  find target -type f -name "*-????????????????" -delete 2>/dev/null || true
fi

# Build broker-server in release mode for CI (saves ~60% space)
# In local dev, use debug mode for faster iteration
if [ -n "$CI" ]; then
  echo "Building broker-server in release mode (CI)"
  cargo build --release --package cmd --bin broker-server
  # Strip debug symbols to save more space (~30% reduction)
  strip target/release/broker-server 2>/dev/null || true
  nohup target/release/broker-server >> 1.log 2>&1 &
else
  echo "Running broker-server in debug mode (local)"
  nohup cargo run --package cmd --bin broker-server >> 1.log 2>&1 &
fi

# Wait a bit for broker to start
sleep 5

# Run tests (still in debug mode for faster compilation)
# cargo nextest run --package grpc-clients --test mod -- meta
# cargo nextest run --package robustmq-test --test mod -- meta
# cargo nextest run --package robustmq-test --test mod -- journal
cargo nextest run --package robustmq-test --test mod -- mqtt

# Clean up broker artifacts after tests (only in CI)
if [ -n "$CI" ]; then
  echo "Cleaning broker artifacts"
  rm -rf target/release/broker-server
  rm -rf target/release/deps/broker_server*
  rm -rf target/release/incremental
fi
