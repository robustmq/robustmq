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
    nohup cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml &
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

start_mqtt_server(){
    nohup cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml &
    sleep 3

    no1=`ps -ef | grep mqtt-server  | grep config/mqtt-server.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "mqtt-server node 1 started successfully. process no: $no1"
    fi
}

stop_mqtt_server(){
    no1=`ps -ef | grep mqtt-server  | grep config/mqtt-server.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "kill mqtt server $no1"
        kill $no1
    fi
}

# Stop Server
stop_placement_server
stop_mqtt_server

# Start Server
start_placement_server
sleep 30
start_mqtt_server
sleep 10

if [ "$1" = "dev" ]; then

  cargo nextest run --package grpc-clients --test mod -- mqtt && \
  cargo nextest run --package robustmq-test --test mod -- mqtt_server && \
  cargo nextest run --package robustmq-test --test mod -- mqtt_protocol

  # Stop Server
  stop_placement_server
  stop_mqtt_server

else
  cargo nextest run --package grpc-clients --test mod -- mqtt && \
  cargo nextest run --package robustmq-test --test mod -- mqtt_server && \
  cargo nextest run --package robustmq-test --test mod -- mqtt_protocol
fi
