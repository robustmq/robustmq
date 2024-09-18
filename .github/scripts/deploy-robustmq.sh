#!/usr/bin/env bash

set -e
set -o pipefail

KUBERNETES_VERSION="${KUBERNETES_VERSION:-v1.24.0}"
ENABLE_STANDALONE_MODE="${ENABLE_STANDALONE_MODE:-true}"
DEFAULT_INSTALL_NAMESPACE=${DEFAULT_INSTALL_NAMESPACE:-default}
ROBUSTMQ_IMAGE_TAG=${ROBUSTMQ_IMAGE_TAG:-latest}
ETCD_CHART="oci://registry-1.docker.io/bitnamicharts/etcd"
GREPTIME_CHART="https://robustteam.github.io/helm-charts/"

# Ceate a cluster with 1 control-plane node and 5 workers.
function create_kind_cluster() {
  cat <<EOF | kind create cluster --name "${CLUSTER}" --image kindest/node:"$KUBERNETES_VERSION" --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
- role: worker
- role: worker
- role: worker
- role: worker
- role: worker
EOF
}

# Add robust Helm chart repo.
function add_robust_chart() {
  helm repo add robust "$GREPTIME_CHART"
  helm repo update
}

# Deploy a etcd cluster with 3 members.
function deploy_etcd_cluster() {
  local namespace="$1"

  helm install etcd "$ETCD_CHART" \
    --set replicaCount=3 \
    --set auth.rbac.create=false \
    --set auth.rbac.token.enabled=false \
    -n "$namespace"

  # Wait for etcd cluster to be ready.
  kubectl rollout status statefulset/etcd -n "$namespace"
}

# Deploy robustmq-operator.
function deploy_robustmq_operator() {
  # Use the latest chart and image.
  helm install robustmq-operator robust/robustmq-operator \
    --set image.tag=latest \
    -n "$DEFAULT_INSTALL_NAMESPACE"

  # Wait for robustmq-operator to be ready.
  kubectl rollout status deployment/robustmq-operator -n "$DEFAULT_INSTALL_NAMESPACE"
}

# Deploy robustmq cluster by using local storage.
# It will expose cluster service ports as '14000', '14001', '14002', '14003' to local access.
function deploy_robustmq_cluster() {
  local cluster_name=$1
  local install_namespace=$2

  kubectl create ns "$install_namespace"

  deploy_etcd_cluster "$install_namespace"

  helm install "$cluster_name" robust/robustmq-cluster \
    --set image.tag="$ROBUSTMQ_IMAGE_TAG" \
    --set meta.etcdEndpoints="etcd.$install_namespace:2379" \
    -n "$install_namespace"

  # Wait for robustmq cluster to be ready.
  while true; do
    PHASE=$(kubectl -n "$install_namespace" get gtc "$cluster_name" -o jsonpath='{.status.clusterPhase}')
    if [ "$PHASE" == "Running" ]; then
      echo "Cluster is ready"
      break
    else
      echo "Cluster is not ready yet: Current phase: $PHASE"
      sleep 5 # wait for 5 seconds before check again.
    fi
  done

  # Expose robustmq cluster to local access.
  kubectl -n "$install_namespace" port-forward svc/"$cluster_name"-frontend \
    14000:4000 \
    14001:4001 \
    14002:4002 \
    14003:4003 > /tmp/connections.out &
}

# Deploy robustmq cluster by using S3.
# It will expose cluster service ports as '24000', '24001', '24002', '24003' to local access.
function deploy_robustmq_cluster_with_s3_storage() {
  local cluster_name=$1
  local install_namespace=$2

  kubectl create ns "$install_namespace"

  deploy_etcd_cluster "$install_namespace"

  helm install "$cluster_name" robust/robustmq-cluster -n "$install_namespace" \
    --set image.tag="$ROBUSTMQ_IMAGE_TAG" \
    --set meta.etcdEndpoints="etcd.$install_namespace:2379" \
    --set storage.s3.bucket="$AWS_CI_TEST_BUCKET" \
    --set storage.s3.region="$AWS_REGION" \
    --set storage.s3.root="$DATA_ROOT" \
    --set storage.credentials.secretName=s3-credentials \
    --set storage.credentials.accessKeyId="$AWS_ACCESS_KEY_ID" \
    --set storage.credentials.secretAccessKey="$AWS_SECRET_ACCESS_KEY"

  # Wait for robustmq cluster to be ready.
  while true; do
    PHASE=$(kubectl -n "$install_namespace" get gtc "$cluster_name" -o jsonpath='{.status.clusterPhase}')
    if [ "$PHASE" == "Running" ]; then
      echo "Cluster is ready"
      break
    else
      echo "Cluster is not ready yet: Current phase: $PHASE"
      sleep 5 # wait for 5 seconds before check again.
    fi
  done

  # Expose robustmq cluster to local access.
  kubectl -n "$install_namespace" port-forward svc/"$cluster_name"-frontend \
    24000:4000 \
    24001:4001 \
    24002:4002 \
    24003:4003 > /tmp/connections.out &
}

# Deploy standalone robustmq.
# It will expose cluster service ports as '34000', '34001', '34002', '34003' to local access.
function deploy_standalone_robustmq() {
  helm install robustmq-standalone robust/robustmq-standalone \
    --set image.tag="$ROBUSTMQ_IMAGE_TAG" \
    -n "$DEFAULT_INSTALL_NAMESPACE"

  # Wait for etcd cluster to be ready.
  kubectl rollout status statefulset/robustmq-standalone -n "$DEFAULT_INSTALL_NAMESPACE"

  # Expose robustmq to local access.
  kubectl -n "$DEFAULT_INSTALL_NAMESPACE" port-forward svc/robustmq-standalone \
    34000:4000 \
    34001:4001 \
    34002:4002 \
    34003:4003 > /tmp/connections.out &
}

# Entrypoint of the script.
function main() {
  create_kind_cluster
  add_robust_chart

  # Deploy standalone robustmq in the same K8s.
  if [ "$ENABLE_STANDALONE_MODE" == "true" ]; then
    deploy_standalone_robustmq
  fi

  deploy_robustmq_operator
  deploy_robustmq_cluster testcluster testcluster
  deploy_robustmq_cluster_with_s3_storage testcluster-s3 testcluster-s3
}

# Usages:
# - Deploy robustmq cluster: ./deploy-robustmq.sh
main
