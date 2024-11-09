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
rm -rf /tmp/robust-test
rm -rf build/
make build

start_server(){
    cd build
    tar -xzvf robustmq-local.tar.gz
    cd ..

    build/robustmq-local/bin/robust-server place start example/mqtt-cluster/placement-center/node-1.toml
    sleep 3
    no1=`ps -ef | grep example/mqtt-cluster/placement-center/node-1.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "placement-center node 1 started successfully. process no: $no1"
    fi

    build/robustmq-local/bin/robust-server journal start example/mqtt-cluster/journal-server/node-1.toml
    sleep 3
    no1=`ps -ef | grep example/mqtt-cluster/journal-server/node-1.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "journal-engine node 1 started successfully. process no: $no1"
    fi

    build/robustmq-local/bin/robust-server mqtt start example/mqtt-cluster/mqtt-server/node-1.toml
    sleep 3
    no1=`ps -ef | grep example/mqtt-cluster/mqtt-server/node-1.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "mqtt-server node 1 started successfully. process no: $no1"
    fi

    sleep 3
}

stop_server(){
    no1=`ps -ef | grep example/mqtt-cluster/placement-center/node-1.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        kill $no1
    fi

    no1=`ps -ef | grep example/mqtt-cluster/journal-server/node-1.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        kill $no1
    fi


    no1=`ps -ef | grep example/mqtt-cluster/mqtt-server/node-1.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        kill $no1
    fi
}

start_server

# Run Cargo Test
cargo nextest run

if [ $? -ne 0 ]; then
    echo "Test case failed to run"
    exit 1
else
    echo "Test case runs successfully"
fi

stop_server

