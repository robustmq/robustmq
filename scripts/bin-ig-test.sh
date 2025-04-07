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

build_server(){
    make build
}

build_server
sleep 5

cd build
tar -xzvf robustmq-0.1.14.tar.gz
cd robustmq-0.1.14
pwd
# bin/robust-server place stop
# bin/robust-server mqtt stop
# bin/robust-server journal stop
# sleep 10

# bin/robust-server place start
# sleep 10
# bin/robust-server mqtt start
# sleep 10
# bin/robust-server journal start

# # place
# cargo nextest run --package grpc-clients --package robustmq-test --test mod -- placement && \
# cargo nextest run --package robustmq-test --test mod -- place_server && \
# cargo nextest run --package storage-adapter --lib -- placement

# # journal
# cargo nextest run  --package grpc-clients --test mod -- journal && \
# cargo nextest run  --package robustmq-test --test mod -- journal_client && \
# cargo nextest run  --package robustmq-test --test mod -- journal_server

# sleep 10
# bin/robust-server place stop
# bin/robust-server mqtt stop
# bin/robust-server journal stop