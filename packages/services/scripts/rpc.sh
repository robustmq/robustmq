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

set -u -e

SCRIPT_DIR="$(dirname "$0")"
BASE_PATH="$SCRIPT_DIR/../../../src/protocol/src"

GENERATE_PATH="$SCRIPT_DIR/../protos"

if [ ! -d "$BASE_PATH" ]; then
    echo "$BASE_PATH does't exist"
    exit 1
fi

# clean the generate path
rm -rf "$GENERATE_PATH"

mkdir -p "$GENERATE_PATH"

proto_dirs_list=""
proto_files=""

while IFS= read -r file; do
  # get all `proto` directories
  if [[ "$file" == *"/proto/"* ]]; then
    proto_dir=$(echo "$file" | sed -E 's/(.*\/proto)\/.*/\1/')

    relative_file_name=$(echo "$file" | sed -E 's/.*\/proto\/(.*)/\1/')
    
    if [[ "$proto_dirs_list" != *"$proto_dir"* ]]; then
      proto_dirs_list="$proto_dirs_list -I=$proto_dir"
    fi

    if [[ "$proto_files" != *"$relative_file_name"* ]]; then
      proto_files="$proto_files $relative_file_name"
    fi
  fi
done < <(find "$BASE_PATH" -type f -name "*.proto")

protoc $proto_dirs_list $proto_files \
  --js_out="import_style=commonjs:$GENERATE_PATH" \
  --grpc-web_out="import_style=typescript,mode=grpcwebtext:$GENERATE_PATH"
