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

stop_pc_cluster(){
    no1=`ps -ef | grep placement-center  | grep node-1 | grep -v grep | awk '{print $2}'`
    if [[ -n $no1 ]]
    then
        echo "kill $no1"
        kill $no1
    fi

    no2=`ps -ef | grep placement-center  | grep node-2 | grep -v grep | awk '{print $2}'`
    if [[ -n $no2 ]]
    then
        echo "kill $no2"
        kill $no2
    fi

    no3=`ps -ef | grep placement-center  | grep node-3 | grep -v grep | awk '{print $2}'`
    if [[ -n $no3 ]]
    then
        echo "kill $no3"
        kill $no3
    fi


    sleep 3

    rm -rf  /tmp/robust/placement-center-1/data
    rm -rf  /tmp/robust/placement-center-2/data
    rm -rf  /tmp/robust/placement-center-3/data
}

stop_pc_cluster