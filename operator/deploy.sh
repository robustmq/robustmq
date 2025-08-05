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


# RobustMQ Operator Deployment Script

set -e

NAMESPACE=${NAMESPACE:-robustmq-system}
IMAGE=${IMAGE:-robustmq/robustmq-operator:latest}

echo "🚀 Deploying RobustMQ Operator..."

# Create namespace if it doesn't exist
echo "📦 Creating namespace ${NAMESPACE}..."
kubectl create namespace ${NAMESPACE} --dry-run=client -o yaml | kubectl apply -f -

# Deploy the operator
echo "🔧 Deploying operator resources..."
sed "s|robustmq/robustmq-operator:latest|${IMAGE}|g" robustmq.yaml | kubectl apply -f -

echo "⏳ Waiting for operator to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/robustmq-operator-controller-manager -n ${NAMESPACE}

echo "✅ RobustMQ Operator deployed successfully!"
echo ""
echo "📋 Next steps:"
echo "1. Deploy a RobustMQ instance:"
echo "   kubectl apply -f sample-robustmq.yaml"
echo ""
echo "2. Check the status:"
echo "   kubectl get robustmq"
echo ""
echo "3. View operator logs:"
echo "   kubectl logs -n ${NAMESPACE} deployment/robustmq-operator-controller-manager -f"
echo ""
echo "🎉 Happy clustering with RobustMQ!"