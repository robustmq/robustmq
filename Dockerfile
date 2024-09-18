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

FROM rust:bullseye as builder

RUN sed -i "s@http://\(deb\|security\).debian.org@https://mirrors.aliyun.com@g" /etc/apt/sources.list && \
    apt-get update && apt-get install -y clang libclang-dev cmake libssl-dev pkg-config

WORKDIR /robustmq

COPY . .

COPY mirror $HOME/.cargo/config.toml

ENV LIBCLANG_PATH=/usr/lib/llvm-11/lib/

RUN cargo build --release && \
    chmod +x 777 ./bin/*

# placement-center
FROM rust:bullseye as placement-center

RUN sed -i "s@http://\(deb\|security\).debian.org@https://mirrors.aliyun.com@g" /etc/apt/sources.list && \
    apt-get update && apt-get install -y clang libclang-dev \

WORKDIR /robustmq

COPY --from=builder bin/* /robustmq/bin/
COPY --from=builder /robustmq/target/release/placement-center /robustmq/libs/placement-center
COPY --from=builder /robustmq/config/* /robustmq/config/

ENTRYPOINT ["bin/robust-server","placement-center","start","config/cluster/placement-center/node.toml"]

# broker-mqtt
FROM rust:bullseye as broker-mqtt

RUN sed -i "s@http://\(deb\|security\).debian.org@https://mirrors.aliyun.com@g" /etc/apt/sources.list && \
    apt-get update && apt-get install -y clang libclang-dev \

WORKDIR /robustmq

COPY --from=builder bin/* /robustmq/bin/
COPY --from=builder /robustmq/target/release/mqtt-server /robustmq/libs/mqtt-server
COPY --from=builder /robustmq/config/* /robustmq/config/

ENTRYPOINT ["bin/robust-server","broker-mqtt","start","config/cluster/mqtt-server/node.toml"]
