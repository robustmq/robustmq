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

make build
sleep 5

cargo clean

cd build
tar -xzvf robustmq-0.1.14.tar.gz
cd robustmq-0.1.14
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
cargo nextest run --package grpc-clients --package robustmq-test --test mod -- placement && \
cargo nextest run --package robustmq-test --test mod -- place_server && \
cargo nextest run --package storage-adapter --lib -- placement

if [ $? -ne 0 ]; then
    echo "place test failed"
    exit 1
fi

# journal
cargo nextest run  --package grpc-clients --test mod -- journal && \
cargo nextest run  --package robustmq-test --test mod -- journal_client && \
cargo nextest run  --package robustmq-test --test mod -- journal_server

if [ $? -ne 0 ]; then
    echo "journal test failed"
    exit 1
fi

# mqtt
cargo nextest run --package grpc-clients --test mod -- mqtt && \
cargo nextest run --package robustmq-test --test mod -- mqtt_server && \
cargo nextest run --package robustmq-test --test mod -- mqtt_protocol

if [ $? -ne 0 ]; then
    echo "mqtt test failed"
    exit 1
fi

sleep 10
bin/robust-server place stop
bin/robust-server mqtt stop
bin/robust-server journal stop