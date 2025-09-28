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
cargo clean

# start server
nohup cargo run --package cmd --bin broker-server >> 1.log 2>&1 &

sleep 30

# place
cargo nextest run --profile ci --package grpc-clients --test mod -- meta
cargo nextest run --profile ci --package robustmq-test --test mod -- meta

if [ $? -ne 0 ]; then
    echo "place test failed"
    exit 1
else
    echo "place test passed"
fi

# journal
cargo nextest run  --profile ci --package robustmq-test --test mod -- journal


if [ $? -ne 0 ]; then
    echo "journal test failed"
    exit 1
else
    echo "journal test passed"
fi

# mqtt
cargo nextest run --profile ci --package robustmq-test --test mod -- mqtt


if [ $? -ne 0 ]; then
    echo "mqtt test failed"
    exit 1
else
    echo "mqtt test passed"
fi
