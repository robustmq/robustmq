# Copyright 2023 RobustMQ Team
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

FROM rust:bullseye AS builder

RUN sed -i "s@http://\(deb\|security\).debian.org@https://mirrors.aliyun.com@g" /etc/apt/sources.list && \
    apt-get update && apt-get install -y clang libclang-dev cmake libssl-dev pkg-config

# download and install protoc
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v23.4/protoc-23.4-linux-x86_64.zip && \
    unzip protoc-23.4-linux-x86_64.zip -d /usr/local && \
    rm protoc-23.4-linux-x86_64.zip

WORKDIR /robustmq

COPY . .

COPY mirror $HOME/.cargo/config.toml

ENV LIBCLANG_PATH=/usr/lib/llvm-11/lib/

RUN cargo build --release && \
    chmod +x  ./bin/*

# placement-center
FROM rust:bullseye AS placement-center

RUN sed -i "s@http://\(deb\|security\).debian.org@https://mirrors.aliyun.com@g" /etc/apt/sources.list && \
    apt-get update && apt-get install -y clang libclang-dev

WORKDIR /robustmq

COPY --from=builder bin/* /robustmq/bin/
COPY --from=builder /robustmq/target/release/placement-center /robustmq/libs/placement-center
COPY --from=builder /robustmq/config/* /robustmq/config/

ENTRYPOINT ["/robustmq/libs/placement-center","--conf","config/cluster/placement-center/node.toml"]

# broker-mqtt
FROM rust:bullseye AS mqtt-server

RUN sed -i "s@http://\(deb\|security\).debian.org@https://mirrors.aliyun.com@g" /etc/apt/sources.list && \
    apt-get update && apt-get install -y clang libclang-dev

WORKDIR /robustmq

COPY --from=builder bin/* /robustmq/bin/
COPY --from=builder /robustmq/target/release/mqtt-server /robustmq/libs/mqtt-server
COPY --from=builder /robustmq/config/* /robustmq/config/

ENTRYPOINT ["/robustmq/libs/mqtt-server","--conf","config/cluster/mqtt-server/node.toml"]

# journal-server
FROM rust:bullseye AS journal-server

RUN sed -i "s@http://\(deb\|security\).debian.org@https://mirrors.aliyun.com@g" /etc/apt/sources.list && \
    apt-get update && apt-get install -y clang libclang-dev

WORKDIR /robustmq

COPY --from=builder bin/* /robustmq/bin/
COPY --from=builder /robustmq/target/release/journal-server /robustmq/libs/journal-server
COPY --from=builder /robustmq/config/* /robustmq/config/

ENTRYPOINT ["/robustmq/libs/journal-server","--conf","config/cluster/journal-server/node.toml"]
