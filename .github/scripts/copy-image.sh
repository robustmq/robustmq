#!/usr/bin/env bash
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


set -e
set -o pipefail

SRC_IMAGE=$1
DST_REGISTRY=$2
SKOPEO_STABLE_IMAGE="quay.io/skopeo/stable:latest"

# Check if necessary variables are set.
function check_vars() {
  for var in DST_REGISTRY_USERNAME DST_REGISTRY_PASSWORD DST_REGISTRY SRC_IMAGE; do
    if [ -z "${!var}" ]; then
      echo "$var is not set or empty."
      echo "Usage: DST_REGISTRY_USERNAME=<your-dst-registry-username> DST_REGISTRY_PASSWORD=<your-dst-registry-password> $0 <dst-registry> <src-image>"
      exit 1
    fi
  done
}

# Copies images from DockerHub to the destination registry.
function copy_images_from_dockerhub() {
  # Check if docker is installed.
  if ! command -v docker &> /dev/null; then
    echo "docker is not installed. Please install docker to continue."
    exit 1
  fi

  # Extract the name and tag of the source image.
  IMAGE_NAME=$(echo "$SRC_IMAGE" | sed "s/.*\///")

  echo "Copying $SRC_IMAGE to $DST_REGISTRY/$IMAGE_NAME"

  docker run "$SKOPEO_STABLE_IMAGE" copy -a docker://"$SRC_IMAGE" \
    --dest-creds "$DST_REGISTRY_USERNAME":"$DST_REGISTRY_PASSWORD" \
    docker://"$DST_REGISTRY/$IMAGE_NAME"
}

function main() {
  check_vars
  copy_images_from_dockerhub
}

# Usage example:
# DST_REGISTRY_USERNAME=123 DST_REGISTRY_PASSWORD=456 \
#   ./copy-image.sh robust/robustmq:v0.4.0 robust-registry.cn-hangzhou.cr.aliyuncs.com
main
