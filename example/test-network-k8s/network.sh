#!/bin/bash
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

export CLUSTER_RUNTIME=robustmq-kind

function context() {
  local name=$1
  local default_value=$2
  local override_name=TEST_NETWORK_${name}
  export ${name}="${!override_name:-${default_value}}"
}

context CLUSTER_NAME                    robustmq-kind
context LOCAL_REGISTRY_NAME             kind-registry
context LOCAL_REGISTRY_INTERFACE        127.0.0.1
context LOCAL_REGISTRY_PORT             5000
context NGINX_HTTP_PORT                 80
context NGINX_HTTPS_PORT                443

context MQTT_SERVER_IMAGE_NAME          mqtt-server-test
context JOURNAL_SERVER_IMAGE_NAME       journal-server-test
context META_SERVICE_IMAGE_NAME     meta-service-test
context CLI_COMMAND_IMAGE_NAME          cli-command-test
context IMAGE_VERSION                   0.2

context NAMESPACE                       robustmq
context STORAGE_CLASS                   standard

source ./scripts/util.sh
source ./scripts/kind.sh
source ./scripts/config.sh
source ./scripts/docker.sh
source ./scripts/cluster.sh
source ./scripts/robustmq.sh
function print_help() {
    echo "Usage: $0 <mode>"
    echo "Modes:"
    echo "  kind       Create KIND cluster"
}


## Parse mode
if [[ $# -lt 1 ]] ; then
  print_help
  exit 0
else
  MODE=$1
  shift
fi

if [ "${MODE}" == "kind" ]; then
  log "Creating KIND cluster \"${CLUSTER_NAME}\":"
  kind_init
  log "🏁 - KIND cluster is ready"
elif [ "${MODE}" == "unkind" ]; then
  kind_unkind
elif [ "${MODE}" == "cluster-init" ]; then
  cluster_init
elif [ "${MODE}" == "build-images" ]; then
  docker_build
elif [ "${MODE}" == "kind-load-images" ]; then
  kind_load_docker_images
elif [ "${MODE}" == "up" ]; then
  network_up
elif [ "${MODE}" == "down" ]; then
  network_down
elif [ "${MODE}" == "cluster-clean" ]; then
cluster_clean
else
  print_help
  exit 1
fi
