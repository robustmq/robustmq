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

start_placement_server(){
    nohup cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml  &
    sleep 3

    no1=`ps -ef | grep placement-center  | grep config/placement-center.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "placement-center node 1 started successfully. process no: $no1"
    fi
}

stop_placement_server(){
    no1=`ps -ef | grep placement-center  | grep config/placement-center.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "kill placement center $no1"
        kill $no1
    fi
}

start_journal_server(){
    nohup cargo run --package cmd --bin journal-server -- --conf=config/journal-server.toml 2>./jn-1.log &
    sleep 3

    no1=`ps -ef | grep journal-server  | grep config/journal-server.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "journal-server node 1 started successfully. process no: $no1"
    fi
}

stop_journal_server(){
    no1=`ps -ef | grep journal-server  | grep config/journal-server.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "kill journal server $no1"
        kill $no1
    fi
}

# Stop Server
stop_placement_server
stop_journal_server

# Start Server
start_placement_server
sleep 30
start_journal_server
sleep 10



if [ "$1" = "dev" ]; then

    cargo nextest run  --package grpc-clients --test mod -- journal && \
    cargo nextest run  --package robustmq-test --test mod -- journal_client && \
    cargo nextest run  --package robustmq-test --test mod -- journal_server

    stop_placement_server
    stop_journal_server

else
    cargo nextest run  --package grpc-clients --test mod -- journal && \
    cargo nextest run  --package robustmq-test --test mod -- journal_client && \
    cargo nextest run  --package robustmq-test --test mod -- journal_server
fi
