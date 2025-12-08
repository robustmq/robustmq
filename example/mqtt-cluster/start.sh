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


sh example/mqtt-cluster/stop.sh

start_server(){

    nohup cargo run --package cmd --bin broker-server -- --conf=example/mqtt-cluster/config/server.toml 2>/tmp/1.log &
    sleep 3

    no=`ps -ef | grep broker-server  | grep example/mqtt-cluster/config/server.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no" ]
    then
        echo "broker server node started successfully. process no: $no"
    fi

    # you can run the following command to query cluster status
    # cargo run --package cmd --bin cli-command status --server 127.0.0.1:8080
}

start_server
