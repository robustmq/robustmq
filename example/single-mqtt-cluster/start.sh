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


sh ./stop.sh

# enable compilation flags
export RUSTFLAGS="--cfg tokio_unstable"

process_logs_dir=process_logs

placement_center_log=./${process_logs_dir}/placement_center.log
mqtt_server_log=./${process_logs_dir}/mqtt_server.log
journal_server_log=./${process_logs_dir}/journal_server.log

if [ -d "$process_logs_dir" ]; then
    echo "mqtt_cluster running logs folder already exists"
else
    echo "mqtt_cluster running logs folder does not exist, creating..."
    mkdir -p "${process_logs_dir}" && echo "mqtt_cluster running logs folder created successfully" || echo "failed to create mqtt_cluster running logs folder"

    echo "\n-------------------------------------\n"
fi

start_pc_cluster(){
    nohup cargo run --package cmd --bin placement-center -- --conf=./placement-center/placement-center.toml > ${placement_center_log} &

    sleep 3

    no1=`ps -ef | grep placement-center | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "placement-center started successfully. process no: $no1"
    fi
    
    echo "\n-------------------------------------\n"

}

start_mqtt_cluster(){
    nohup cargo run --package cmd --bin mqtt-server -- --conf=./mqtt-server/mqtt-server.toml > ${mqtt_server_log} &
    
    no1=`ps -ef | grep mqtt-server | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "mqtt cluster node 1 started successfully. process no: $no1"
    fi

    echo "\n-------------------------------------\n"

}

start_journal_cluster(){
    nohup cargo run --package cmd --bin journal-server -- --conf=./journal-server/journal-server.toml > ${journal_server_log} &
    
    no1=`ps -ef | grep journal-server | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "journal-server node 1 started successfully. process no: $no1"
    fi

    echo "\n-------------------------------------\n"
}

start_pc_cluster

sleep 3

start_journal_cluster

sleep 3

start_mqtt_cluster
