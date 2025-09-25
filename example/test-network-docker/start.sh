#!/bin/bash
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

cd ../../
docker build --target builder -t builder-test:0.1 .
docker build --target meta-service -t meta-service-test:0.1 .
docker build --target mqtt-server -t mqtt-server-test:0.1 .
docker build --target journal-server -t journal-server-test:0.1 .

cd ./example/test-network-docker

docker-compose -f compose/compose-test-net.yaml up
