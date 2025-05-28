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

function kind_create() {

  kind delete cluster --name ${CLUSTER_NAME}

  local reg_name=${LOCAL_REGISTRY_NAME}
  local reg_port=${LOCAL_REGISTRY_PORT}
  local ingress_http_port=${NGINX_HTTP_PORT}
  local ingress_https_port=${NGINX_HTTPS_PORT}

  # the 'ipvs'proxy mode permits better HA abilities

  cat <<EOF | kind create cluster --name ${CLUSTER_NAME} --config=-
---
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
    kubeadmConfigPatches:
      - |
        kind: InitConfiguration
        nodeRegistration:
          kubeletExtraArgs:
            node-labels: "ingress-ready=true"
    extraPortMappings:
      - containerPort: 80
        hostPort: ${ingress_http_port}
        protocol: TCP
      - containerPort: 443
        hostPort: ${ingress_https_port}
        protocol: TCP
#networking:
#  kubeProxyMode: "ipvs"

# create a cluster with the local registry enabled in containerd
containerdConfigPatches:
- |-
  [plugins."io.containerd.grpc.v1.cri".registry.mirrors."localhost:${reg_port}"]
    endpoint = ["http://${reg_name}:${reg_port}"]

EOF

  for node in $(kind get nodes --name "${CLUSTER_NAME}");
  do
    #  Kind 节点容器中启用路由回环，以便让容器节点可以通过网络访问主机的回环地址（localhost），从而实现一些特定的测试或网络通信需求。
    docker exec "$node" sysctl net.ipv4.conf.all.route_localnet=1;
  done

}


function kind_load_docker_images() {
    local mqtt_server_tag=${MQTT_SERVER_IMAGE_NAME}:${IMAGE_VERSION}
    local placement_center_tag=${PLACEMENT_CENTER_IMAGE_NAME}:${IMAGE_VERSION}
    local cli_command_tag=${CLI_COMMAND_IMAGE_NAME}:${IMAGE_VERSION}
    # 加载的镜像只在当前 Kind 集群有效。如果你销毁并重新创建集群，需要重新加载镜像。
    kind load docker-image ${mqtt_server_tag}      --name ${CLUSTER_NAME}
    kind load docker-image ${placement_center_tag} --name ${CLUSTER_NAME}
    kind load docker-image ${cli_command_tag}     --name ${CLUSTER_NAME}

    kind load docker-image k8s.gcr.io/ingress-nginx/kube-webhook-certgen:v1.1.1 --name ${CLUSTER_NAME}
    kind load docker-image k8s.gcr.io/ingress-nginx/controller:v1.1.2 --name ${CLUSTER_NAME}
    kind load docker-image quay.io/jetstack/cert-manager-webhook:v1.6.1 --name ${CLUSTER_NAME}
    kind load docker-image quay.io/jetstack/cert-manager-controller:v1.6.1 --name ${CLUSTER_NAME}
    kind load docker-image quay.io/jetstack/cert-manager-cainjector:v1.6.1 --name ${CLUSTER_NAME}

}

function launch_docker_registry() {

  # 创建注册表容器，除非它已经存在
  local reg_name=${LOCAL_REGISTRY_NAME}
  local reg_port=${LOCAL_REGISTRY_PORT}
  local reg_interface=${LOCAL_REGISTRY_INTERFACE}

  # 作为一个私有 Docker 镜像仓库，监听本地主机的 5000 端口，提供镜像存储服务。
  running="$(docker inspect -f '{{.State.Running}}' "${reg_name}" 2>/dev/null || true)"
  if [ "${running}" != 'true' ]; then
    docker run  \
      --detach  \
      --restart always \
      --name    "${reg_name}" \
      --publish "${reg_interface}:${reg_port}:5000" \
      registry:2
  fi
  # 此操作将容器 ${reg_name} 加入到 Docker 网络 kind 中。随后，Kind 集群中的节点可以通过 ${reg_name} 容器名直接访问私有镜像仓库，而无需通过宿主机端口。
  # Kind 创建集群时，会自动在 Docker 中生成一个默认的网络，名字固定为 kind。
  docker network connect "kind" "${reg_name}" || true

  cat <<EOF | kubectl apply -f -
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: local-registry-hosting
  namespace: kube-public
data:
  localRegistryHosting.v1: |
    host: "localhost:${reg_port}"
    help: "https://kind.sigs.k8s.io/docs/user/local-registry/"
EOF


}


function stop_docker_registry() {
  docker kill kind-registry || true
  docker rm kind-registry   || true
}

function kind_delete() {
  kind delete cluster --name ${CLUSTER_NAME}
}

function kind_init() {
  kind_create
  launch_docker_registry
}

function kind_unkind() {
  kind_delete
  stop_docker_registry
}
