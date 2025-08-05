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


# RobustMQ Operator Undeployment Script

set -e

NAMESPACE=${NAMESPACE:-robustmq-system}

echo "🗑️  Undeploying RobustMQ Operator..."

# Delete all RobustMQ instances first
echo "🔍 Checking for existing RobustMQ instances..."
if kubectl get robustmq --all-namespaces --no-headers 2>/dev/null | grep -q .; then
    echo "⚠️  Found existing RobustMQ instances. Please delete them first:"
    kubectl get robustmq --all-namespaces
    echo ""
    echo "Run the following to delete all instances:"
    echo "kubectl delete robustmq --all --all-namespaces"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Delete the operator
echo "🔧 Deleting operator resources..."
kubectl delete -f robustmq.yaml --ignore-not-found=true

# Optionally delete the namespace
echo ""
read -p "Delete namespace ${NAMESPACE}? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📦 Deleting namespace ${NAMESPACE}..."
    kubectl delete namespace ${NAMESPACE} --ignore-not-found=true
fi

echo "✅ RobustMQ Operator undeployed successfully!"
echo ""
echo "📋 Manual cleanup (if needed):"
echo "1. Check for remaining resources:"
echo "   kubectl get robustmq --all-namespaces"
echo "   kubectl get crd | grep robustmq"
echo ""
echo "2. Clean up CRDs (if needed):"
echo "   kubectl delete crd robustmqs.robustmq.io"
echo ""
echo "🎉 Cleanup complete!"