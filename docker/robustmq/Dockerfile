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

# ==============================================================================
# Build Stage - Using cargo-chef for better dependency caching
# ==============================================================================

# Stage 1: Planner - Generate dependency recipe
FROM lukemathwalker/cargo-chef:latest-rust-1.90.0 AS planner

WORKDIR /robustmq

# Copy all Cargo files to generate recipe
# --parents preserves directory structure (requires BuildKit 0.13+)
COPY --parents Cargo.toml Cargo.lock \
    src/*/Cargo.toml \
    src/common/*/Cargo.toml \
    src/protocol/build.rs \
    tests/Cargo.toml \
    ./

# Auto-create dummy src/lib.rs or src/main.rs for every crate found via Cargo.toml
RUN find . -name "Cargo.toml" -not -path "./Cargo.toml" -exec dirname {} \; | while read dir; do \
      mkdir -p "$dir/src"; \
      if grep -q '\[\[bin\]\]' "$dir/Cargo.toml" 2>/dev/null; then \
        echo 'fn main() {}' > "$dir/src/main.rs"; \
      fi; \
      echo '' > "$dir/src/lib.rs"; \
    done

# Generate recipe.json with full dependency graph
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - Compile dependencies and application
FROM rust:1.90.0-bookworm AS builder

WORKDIR /robustmq

# Install system dependencies
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    cmake \
    libssl-dev \
    pkg-config \
    ca-certificates \
    curl \
    lld \
    && rm -rf /var/lib/apt/lists/*

# Install protoc (Protocol Buffers compiler) - auto-detect architecture
ARG PROTOC_VERSION=23.4
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
        x86_64)  PROTOC_ARCH="linux-x86_64" ;; \
        aarch64) PROTOC_ARCH="linux-aarch_64" ;; \
        *) echo "Unsupported arch: $ARCH" && exit 1 ;; \
    esac && \
    curl -LO "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-${PROTOC_ARCH}.zip" \
    && unzip "protoc-${PROTOC_VERSION}-${PROTOC_ARCH}.zip" -d /usr/local \
    && rm "protoc-${PROTOC_VERSION}-${PROTOC_ARCH}.zip"

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef --locked

# Set environment variables for compilation
# Find LLVM library path - try multiple common locations
RUN LIBCLANG_PATH=$(find /usr/lib -name "libclang*" -type f 2>/dev/null | head -1 | xargs dirname 2>/dev/null || \
                    find /usr/lib/x86_64-linux-gnu -name "libclang*" -type f 2>/dev/null | head -1 | xargs dirname 2>/dev/null || \
                    echo "/usr/lib/llvm-14/lib") && \
    echo "LIBCLANG_PATH=$LIBCLANG_PATH" >> /etc/environment
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib/ \
    PATH="/usr/local/bin:${PATH}"

# Copy dependency recipe from planner
COPY --from=planner /robustmq/recipe.json recipe.json

# Build dependencies - this layer is cached unless Cargo.lock changes
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the actual source code
COPY . .

# Build the application (dependencies are already compiled)
RUN cargo build --release \
    && strip target/release/broker-server \
    && strip target/release/cli-command \
    && strip target/release/cli-bench

# ==============================================================================
# Runtime Stage
# ==============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create robustmq user and group
RUN groupadd -r robustmq && useradd -r -g robustmq robustmq

# Create necessary directories
RUN mkdir -p /robustmq/{bin,libs,config,data,logs} \
    && chown -R robustmq:robustmq /robustmq

# Set working directory
WORKDIR /robustmq

# Copy binaries from builder
COPY --from=builder /robustmq/target/release/broker-server ./libs/
COPY --from=builder /robustmq/target/release/cli-command ./libs/
COPY --from=builder /robustmq/target/release/cli-bench ./libs/

# Copy configuration files and scripts
COPY --chown=robustmq:robustmq config/ ./config/
COPY --chown=robustmq:robustmq bin/ ./bin/

# Make scripts executable
RUN chmod +x ./bin/*

# Create default configuration if not exists
RUN if [ ! -f ./config/server.toml ]; then \
        cp ./config/server.toml.template ./config/server.toml 2>/dev/null || true; \
    fi

# Switch to non-root user
USER robustmq

# Expose ports
# MQTT: 1883 (TCP), 1885 (SSL), 8083 (WebSocket), 8085 (WSS)
# Kafka: 9092
# gRPC: 1228
# AMQP: 5672
EXPOSE 1883 1885 8083 8085 9092 1228 5672 8080

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/api/status || curl -f http://localhost:8080/ || exit 1

# Default environment variables
ENV ROBUSTMQ_CONFIG_PATH="/robustmq/config/server.toml"
ENV ROBUSTMQ_LOG_PATH="/robustmq/logs"
ENV ROBUSTMQ_DATA_PATH="/robustmq/data"

# Entry point
ENTRYPOINT ["./libs/broker-server"]
CMD ["--conf=/robustmq/config/server.toml"]

# ==============================================================================
# Named stages for docker-compose target selection
# ==============================================================================

# Meta Service
FROM runtime AS meta-service
EXPOSE 1228 1227
CMD ["--conf=/robustmq/config/server.toml"]

# MQTT Broker
FROM runtime AS mqtt-broker
EXPOSE 1883 1885 8083 8085 8080
CMD ["--conf=/robustmq/config/server.toml"]

# All-in-one (default)
FROM runtime AS all-in-one
CMD ["--conf=/robustmq/config/server.toml"]
