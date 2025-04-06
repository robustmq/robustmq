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

check_status() {
   make > /dev/null
   if [ $? -ne 0 ]; then
      echo "Error: Make command failed"
      return 1
   fi
   cd build
   matched_file=$(ls | grep -E '^robustmq-[0-9]+\.[0-9]+\.[0-9]+\.tar\.gz$')
   tar -xzf "$matched_file" > /dev/null
    if [ $? -ne 0 ]; then
        echo "Error: Failed to extract the tarball."
        return 1
    fi

    cd robustmq-*/bin

    place_status=$(./robust-ctl place status)
    if [ $? -ne 0 ]; then
        echo "Error: Failed to get the status of placement center."
        return 1
    fi

    while true; do
      if [[ "$place_status" == *'"running_state":{"Ok":null}'* ]]; then
        echo "Placement center is ready now!"
        break
      else
        echo "Placement center starts but not ready yet. Waiting..."
        sleep 2
      fi
    done
}
