#!/bin/bash

# Build and start broker
cargo build --release --package cmd --bin broker-server
strip target/release/broker-server 2>/dev/null || true
nohup target/release/broker-server >> 1.log 2>&1 &
BROKER_PID=$!
sleep 5

# Run tests
cargo nextest run --package grpc-clients --test mod -- meta
cargo nextest run --package robustmq-test --test mod -- meta
cargo nextest run --package robustmq-test --test mod -- journal
cargo nextest run --package robustmq-test --test mod -- mqtt

# Stop broker
kill $BROKER_PID 2>/dev/null || true
