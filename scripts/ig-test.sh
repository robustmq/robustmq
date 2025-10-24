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


# Build and start broker
cargo build --release --package cmd --bin broker-server
strip target/release/broker-server 2>/dev/null || true
nohup target/release/broker-server >> 1.log 2>&1 &
BROKER_PID=$!
sleep 30

# Run all tests in one command to avoid repeated compilation
cargo nextest run --release \
  --package grpc-clients \
  --package robustmq-test

# Stop broker
kill $BROKER_PID 2>/dev/null || true
