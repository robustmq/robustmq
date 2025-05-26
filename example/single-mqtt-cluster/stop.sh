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

dir_clean(){
  local target_dir=$1

  if [ -d "$target_dir" ]
  then
    rm -rf ${target_dir}
    echo "clean up related tmp directories: ${target_dir}"
  fi
}

pre_clean(){
  remove_dirs=(
    "./logs"
    "./robust-data-test"
    "./process_logs"
  )

  for remove_dir in "${remove_dirs[@]}"; do
    dir_clean "$remove_dir"
  done
}

stop_pc_cluster(){
    no1=`ps -ef | grep placement-center | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "kill placement center $no1"
        kill $no1
    fi
}

stop_mqtt_cluster(){
    no1=`ps -ef | grep mqtt-server | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "kill mqtt-server $no1"
        kill $no1
    fi
}

stop_journal_server(){
    no1=`ps -ef | grep journal-server | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "kill journal-server $no1"
        kill $no1
    fi
}

pre_clean

stop_pc_cluster

stop_journal_server

stop_mqtt_cluster

# Simply wait for the process to exit
sleep 3