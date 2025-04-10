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

make build
sleep 5

cargo clean

cd build
tar -xzvf robustmq-${VERSION}.tar.gz
cd robustmq-${VERSION}
pwd
bin/robust-server place stop
bin/robust-server mqtt stop
bin/robust-server journal stop

sleep 10
bin/robust-server place start
sleep 10
bin/robust-server mqtt start
sleep 10
bin/robust-server journal start
sleep 10

# place
cargo nextest run --profile ci --package grpc-clients --package robustmq-test --test mod -- placement && \
cargo nextest run --profile ci --package robustmq-test --test mod -- place_server

if [ $? -ne 0 ]; then
    echo "place test failed"
    exit 1
else
    echo "place test passed"
fi

cargo clean

# journal
cargo nextest run  --profile ci --package grpc-clients --test mod -- journal && \
cargo nextest run  --profile ci --package robustmq-test --test mod -- journal_client && \
cargo nextest run  --profile ci --package robustmq-test --test mod -- journal_server

if [ $? -ne 0 ]; then
    echo "journal test failed"
    exit 1
else
    echo "journal test passed"
fi

cargo clean
# mqtt
cargo nextest run --profile ci --package grpc-clients --test mod -- mqtt && \
cargo nextest run --profile ci --package robustmq-test --test mod -- mqtt_server && \
cargo nextest run --profile ci --package robustmq-test --test mod -- mqtt_protocol

if [ $? -ne 0 ]; then
    echo "mqtt test failed"
    exit 1
else
    echo "mqtt test passed"
fi

cargo clean
# storage-adapter
cargo nextest run --profile ci --package storage-adapter --lib -- placement

sleep 10
bin/robust-server place stop
bin/robust-server mqtt stop
bin/robust-server journal stop