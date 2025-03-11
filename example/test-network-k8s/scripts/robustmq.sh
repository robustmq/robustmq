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


function network_up() {
    kubectl create namespace ${NAMESPACE}
    apply_template kube/cfg-placement-center.yaml ${NAMESPACE}
    apply_template kube/placement-center.yaml ${NAMESPACE}
    apply_template kube/cfg-mqtt-server.yaml ${NAMESPACE}
    apply_template kube/mqtt-server.yaml ${NAMESPACE}
    apply_template kube/mqtt-server-ingress.yaml ${NAMESPACE}
    apply_template kube/cli-command.yaml ${NAMESPACE}
}

function network_down() {
    kubectl delete deployment mqtt-server -n ${NAMESPACE}
    kubectl delete statefulset placement-center -n ${NAMESPACE}
    kubectl delete configmap placement-center-config -n ${NAMESPACE}
    kubectl delete configmap mqtt-server-config -n ${NAMESPACE}
    kubectl delete deployment cli-command -n ${NAMESPACE}
    kubectl delete namespace ${NAMESPACE}
#    kubectl delete ingress mqtt-server-ingress -n ${NAMESPACE}
}
