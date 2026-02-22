# Kubernetes Operator

RobustMQ Operator is a Go-based Kubernetes controller that manages RobustMQ cluster deployment and scaling via a custom CRD (`robustmq.io/v1alpha1`).

> **Note**: The Operator is in early development. Recommended for testing and evaluation.

---

## Prerequisites

- Kubernetes 1.20+
- kubectl configured with cluster access

---

## Install Operator

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq/operator

# Deploy CRD + Operator
kubectl apply -f robustmq.yaml

# Wait until ready
kubectl wait --for=condition=available --timeout=300s \
  deployment/robustmq-operator-controller-manager -n robustmq-system

# Verify
kubectl get pods -n robustmq-system
kubectl get crd | grep robustmq
```

---

## Deploy an Instance

### AllInOne (Dev / Test)

```bash
kubectl apply -f sample-robustmq.yaml
```

Or with a custom manifest:

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-dev
  namespace: default
spec:
  deploymentMode: AllInOne
  image:
    repository: ghcr.io/robustmq/robustmq
    tag: latest
  allInOne:
    replicas: 1
    resources:
      requests:
        memory: "1Gi"
        cpu: "500m"
      limits:
        memory: "2Gi"
        cpu: "1000m"
  storage:
    storageClass: standard
    size: "10Gi"
  monitoring:
    enabled: true
```

### Microservices (Production)

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-prod
  namespace: default
spec:
  deploymentMode: Microservices
  image:
    repository: ghcr.io/robustmq/robustmq
    tag: latest
  microservices:
    metaService:
      replicas: 3
    brokerService:
      replicas: 2
  storage:
    storageClass: fast-ssd
    size: "20Gi"
  network:
    mqtt:
      serviceType: LoadBalancer
  monitoring:
    enabled: true
```

---

## Check Status

```bash
# List instances
kubectl get robustmq

# Pods
kubectl get pods -l app=robustmq

# Services
kubectl get svc -l app=robustmq

# Operator logs
kubectl logs -n robustmq-system \
  deployment/robustmq-operator-controller-manager -f
```

---

## Scale

```bash
kubectl patch robustmq robustmq-dev --type='merge' \
  -p='{"spec":{"allInOne":{"replicas":3}}}'
```

---

## Uninstall

```bash
# Remove instances
kubectl delete robustmq --all --all-namespaces

# Remove Operator
kubectl delete -f robustmq.yaml
```
