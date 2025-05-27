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

SCRIPT_PATH="$(greadlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname "$SCRIPT_PATH")"
cd "$SCRIPT_DIR/../"

source "scripts/check-place-status.sh"
source "scripts/stop-place.sh"

start_server(){
    nohup cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml 2>/tmp/1.log &
    sleep 3

    no1=`ps -ef | grep placement-center  | grep config/placement-center.toml | grep -v grep | awk '{print $2}'`
    if [ -n "$no1" ]
    then
        echo "placement-center node 1 started successfully. process no: $no1"
    fi
}

# Stop server if it is running
stop_server

# Start a new server
start_server

# Wait until the server is ready
check_status
