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

# Run Cargo Test
cargo nextest run \
 -p amqp-broker \
 -p amqp-plugins \
 -p cli-command \
 -p common-base \
 -p metadata-struct \
 -p rocksdb-engine \
 -p third-driver \
 -p journal-client \
 -p journal-server \
 -p placement-center \
 -p protocol \
 -p storage-adapter

 # Modules that are not suitable for unit testing
 # cmd
 # grpc-clients
 # mqtt-bridge
 # mqtt-broker