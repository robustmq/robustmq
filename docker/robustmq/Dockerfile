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
# Downloader Stage - Pull pre-built release package
# ==============================================================================
FROM debian:bookworm-slim AS downloader

WORKDIR /robustmq

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    tar \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Required: VERSION must be like v0.3.3
ARG VERSION
ARG TARGETARCH
ARG GITHUB_REPO=robustmq/robustmq

# Poll release asset for up to 40 minutes, checking every 60 seconds.
# Log each attempt so CI output clearly shows wait progress.
RUN set -e; \
    if [ -z "${VERSION}" ]; then \
      echo "ERROR: build arg VERSION is required (example: --build-arg VERSION=v0.3.3)"; \
      exit 1; \
    fi; \
    case "${TARGETARCH}" in \
      amd64|arm64) ASSET_ARCH="${TARGETARCH}" ;; \
      *) echo "ERROR: unsupported TARGETARCH=${TARGETARCH}"; exit 1 ;; \
    esac; \
    ASSET_NAME="robustmq-${VERSION}-linux-${ASSET_ARCH}.tar.gz"; \
    ASSET_URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${ASSET_NAME}"; \
    echo "INFO: waiting for release asset: ${ASSET_URL}"; \
    ATTEMPT=1; \
    MAX_ATTEMPTS=40; \
    while [ "${ATTEMPT}" -le "${MAX_ATTEMPTS}" ]; do \
      HTTP_CODE="$(curl -sSL -o /dev/null -w "%{http_code}" "${ASSET_URL}" || true)"; \
      if [ "${HTTP_CODE}" = "200" ]; then \
        echo "INFO: asset is available (attempt ${ATTEMPT}/${MAX_ATTEMPTS}), downloading..."; \
        curl -fsSLo "/tmp/${ASSET_NAME}" "${ASSET_URL}"; \
        mkdir -p /tmp/extract; \
        tar -xzf "/tmp/${ASSET_NAME}" -C /tmp/extract; \
        PKG_DIR="/tmp/extract/robustmq-${VERSION}-linux-${ASSET_ARCH}"; \
        if [ ! -d "${PKG_DIR}" ]; then \
          echo "ERROR: extracted package directory not found: ${PKG_DIR}"; \
          exit 1; \
        fi; \
        mkdir -p /robustmq/dist; \
        cp -r "${PKG_DIR}/bin" /robustmq/; \
        cp -r "${PKG_DIR}/libs" /robustmq/; \
        cp -r "${PKG_DIR}/config" /robustmq/; \
        if [ -d "${PKG_DIR}/dist" ]; then cp -r "${PKG_DIR}/dist" /robustmq/; fi; \
        if [ -f "${PKG_DIR}/LICENSE" ]; then cp "${PKG_DIR}/LICENSE" /robustmq/; else touch /robustmq/LICENSE; fi; \
        if [ -f "${PKG_DIR}/package-info.txt" ]; then cp "${PKG_DIR}/package-info.txt" /robustmq/; else touch /robustmq/package-info.txt; fi; \
        echo "INFO: release package prepared successfully"; \
        exit 0; \
      fi; \
      echo "INFO: asset not ready yet (attempt ${ATTEMPT}/${MAX_ATTEMPTS}, http=${HTTP_CODE})"; \
      if [ "${ATTEMPT}" -ge "${MAX_ATTEMPTS}" ]; then break; fi; \
      echo "INFO: retrying in 60s..."; \
      ATTEMPT=$((ATTEMPT + 1)); \
      sleep 60; \
    done; \
    echo "ERROR: timed out after 40 minutes waiting for ${ASSET_URL}"; \
    exit 1

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

# Copy packaged artifacts from downloader
COPY --from=downloader --chown=robustmq:robustmq /robustmq/libs ./libs
COPY --from=downloader --chown=robustmq:robustmq /robustmq/bin ./bin
COPY --from=downloader --chown=robustmq:robustmq /robustmq/config ./config
COPY --from=downloader --chown=robustmq:robustmq /robustmq/dist ./dist
COPY --from=downloader --chown=robustmq:robustmq /robustmq/LICENSE ./LICENSE
COPY --from=downloader --chown=robustmq:robustmq /robustmq/package-info.txt ./package-info.txt

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
