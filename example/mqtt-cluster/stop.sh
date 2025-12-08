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

stop_cluster(){
    pids=$(ps -ef | grep broker-server | grep -v grep | awk '{print $2}')
    
    if [ -n "$pids" ]; then
        for pid in $pids; do
            echo "kill broker server process: $pid"
            kill "$pid"
        done
    fi

    sleep 3
    rm -rf example/mqtt-cluster/data
    rm -rf example/mqtt-cluster/nohup.out
}

stop_cluster
