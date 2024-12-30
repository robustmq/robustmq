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


function cluster_init() {

  apply_nginx_ingress
  apply_cert_manager

  sleep 2

  wait_for_cert_manager
  wait_for_nginx_ingress

  # if [ "${STAGE_DOCKER_IMAGES}" == true ]; then
  #   pull_docker_images
  #   kind_load_docker_images
  # fi
}

function apply_nginx_ingress() {

  # 1.1.2 static ingress with modifications to enable ssl-passthrough
  # k3s : 'cloud'
  # kind : 'kind'
  # kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.1.2/deploy/static/provider/cloud/deploy.yaml

  kubectl apply -f kube/ingress-nginx-${CLUSTER_RUNTIME}.yaml

}


function delete_nginx_ingress() {

  cat kube/ingress-nginx-${CLUSTER_RUNTIME}.yaml | kubectl delete -f -

}

function wait_for_nginx_ingress() {
  kubectl wait --namespace ingress-nginx \
    --for=condition=ready pod \
    --selector=app.kubernetes.io/component=controller \
    --timeout=2m
}

function apply_cert_manager() {
  # Install cert-manager to manage TLS certificates
  kubectl apply -f kube/cert-manager.yaml
}

function delete_cert_manager() {
  # Install cert-manager to manage TLS certificates
  curl kube/cert-manager.yaml | kubectl delete -f -
}

function wait_for_cert_manager() {

  kubectl -n cert-manager rollout status deploy/cert-manager
  kubectl -n cert-manager rollout status deploy/cert-manager-cainjector
  kubectl -n cert-manager rollout status deploy/cert-manager-webhook

}

function cluster_clean() {
  delete_nginx_ingress
  delete_cert_manager
}
