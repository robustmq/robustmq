# K8S Operator 部署

RobustMQ Operator 是基于 Go 实现的 Kubernetes 控制器，通过自定义 CRD（`robustmq.io/v1alpha1`）管理 RobustMQ 集群的部署与扩缩容。

> **注意**：Operator 目前处于早期开发阶段，建议用于测试和评估。

---

## 前置条件

- Kubernetes 1.20+
- kubectl 已配置集群访问

---

## 安装 Operator

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq/operator

# 部署 CRD + Operator
kubectl apply -f robustmq.yaml

# 等待就绪
kubectl wait --for=condition=available --timeout=300s \
  deployment/robustmq-operator-controller-manager -n robustmq-system

# 验证
kubectl get pods -n robustmq-system
kubectl get crd | grep robustmq
```

---

## 部署实例

### AllInOne（开发 / 测试）

```bash
kubectl apply -f sample-robustmq.yaml
```

也可以用自定义 YAML：

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

### Microservices（生产）

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

## 查看状态

```bash
# 查看实例列表和状态
kubectl get robustmq

# 查看 Pod
kubectl get pods -l app=robustmq

# 查看服务
kubectl get svc -l app=robustmq

# 查看 Operator 日志
kubectl logs -n robustmq-system \
  deployment/robustmq-operator-controller-manager -f
```

---

## 扩缩容

```bash
kubectl patch robustmq robustmq-dev --type='merge' \
  -p='{"spec":{"allInOne":{"replicas":3}}}'
```

---

## 卸载

```bash
# 删除实例
kubectl delete robustmq --all --all-namespaces

# 删除 Operator
kubectl delete -f robustmq.yaml
```
