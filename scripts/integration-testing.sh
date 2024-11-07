#!/bin/sh
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
rm -rf /home/runner/work/robustmq/robustmq/target

sh example/mqtt-cluster/start.sh
sleep 10

# Run Cargo Test
cargo nextest run

if [ $? -ne 0 ]; then
    echo "Test case failed to run"
    sh example/mqtt-cluster/stop.sh
    exit 1
else
    echo "Test case runs successfully"
    sh example/mqtt-cluster/stop.sh
fi

