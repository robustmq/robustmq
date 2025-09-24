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
# Build Stage
# ==============================================================================
FROM rust:1.70-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    cmake \
    libssl-dev \
    pkg-config \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install protoc (Protocol Buffers compiler)
ARG PROTOC_VERSION=23.4
RUN curl -LO "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip" \
    && unzip "protoc-${PROTOC_VERSION}-linux-x86_64.zip" -d /usr/local \
    && rm "protoc-${PROTOC_VERSION}-linux-x86_64.zip"

# Set environment variables for compilation
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib/
ENV PATH="/usr/local/bin:${PATH}"

# Create app directory
WORKDIR /robustmq

# Copy dependency files first for better cache utilization
COPY Cargo.toml Cargo.lock ./
COPY src/*/Cargo.toml ./src/*/
COPY src/common/*/Cargo.toml ./src/common/*/
COPY tests/Cargo.toml ./tests/

# Create dummy source files to build dependencies
RUN find src -name "Cargo.toml" -exec dirname {} \; | xargs -I {} mkdir -p {}/src
RUN find src -name "Cargo.toml" -exec dirname {} \; | xargs -I {} touch {}/src/lib.rs

# Build dependencies (this will be cached unless Cargo.toml changes)
RUN cargo build --release || true

# Remove dummy source files
RUN find . -name "*.rs" -delete

# Copy the actual source code
COPY . .

# Build the application
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
# MQTT: 1883 (TCP), 1884 (SSL), 8083 (WebSocket), 8084 (WSS)
# Kafka: 9092
# gRPC: 1228
# AMQP: 5672
EXPOSE 1883 1884 8083 8084 9092 1228 5672 8080

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/api/status || exit 1

# Default environment variables
ENV ROBUSTMQ_CONFIG_PATH="/robustmq/config/server.toml"
ENV ROBUSTMQ_LOG_PATH="/robustmq/logs"
ENV ROBUSTMQ_DATA_PATH="/robustmq/data"

# Entry point
ENTRYPOINT ["./libs/broker-server"]
CMD ["--conf=/robustmq/config/server.toml"]

# ==============================================================================
# Alternative: Service-specific stages for microservice deployment
# ==============================================================================

# Meta Service (Placement Center)
FROM runtime AS meta-service
ENV ROBUSTMQ_ROLES="meta"
EXPOSE 1228
CMD ["--conf=/robustmq/config/server.toml"]

# MQTT Broker Service
FROM runtime AS mqtt-broker
ENV ROBUSTMQ_ROLES="broker"
EXPOSE 1883 1884 8083 8084 8080
CMD ["--conf=/robustmq/config/server.toml"]

# Journal Service
FROM runtime AS journal-service
ENV ROBUSTMQ_ROLES="journal"
EXPOSE 1228
CMD ["--conf=/robustmq/config/server.toml"]

# All-in-one (default)
FROM runtime AS all-in-one
ENV ROBUSTMQ_ROLES="meta,broker,journal"
CMD ["--conf=/robustmq/config/server.toml"]

# ==============================================================================
# Development Stage (for development and debugging)
# ==============================================================================
FROM builder AS development

# Install additional development tools
RUN apt-get update && apt-get install -y \
    gdb \
    strace \
    valgrind \
    && rm -rf /var/lib/apt/lists/*

# Build debug version
RUN cargo build

# Copy configuration
COPY config/ ./config/
COPY bin/ ./bin/
RUN chmod +x ./bin/*

EXPOSE 1883 1884 8083 8084 9092 1228 5672 8080

# Development entry point
ENTRYPOINT ["cargo", "run", "--package", "cmd", "--bin", "broker-server", "--"]
CMD ["--conf=./config/server.toml"]