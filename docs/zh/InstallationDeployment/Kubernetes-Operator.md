# Kubernetes Operator 部署

本指南介绍如何使用 RobustMQ Operator 在 Kubernetes 环境中部署和管理 RobustMQ 集群。

## 概述

RobustMQ Operator 是一个 Kubernetes 控制器，用于自动化 RobustMQ 集群的部署、扩展和管理。它提供了声明式的 API 来管理 RobustMQ 实例，支持多种部署模式和高级功能。

### 主要特性

- **部署模式**: 支持 AllInOne 和 Microservices 两种部署模式
- **自动扩缩容**: 自动管理 RobustMQ 组件的副本数量
- **配置管理**: 自动生成和管理 RobustMQ 配置
- **多协议支持**: MQTT、Kafka、gRPC 和 AMQP 协议支持
- **监控集成**: 内置 Prometheus 监控和 ServiceMonitor 支持
- **安全功能**: TLS 加密和身份验证支持
- **存储管理**: 持久卷管理用于数据存储
- **高可用性**: 支持多副本部署

## 准备工作

### 系统要求

- **Kubernetes**: 版本 1.20+
- **kubectl**: 配置为访问您的集群
- **Helm**: 版本 3.x（可选，用于基于 Helm 的安装）

### 检查集群状态

```bash
# 检查集群连接
kubectl cluster-info

# 检查节点状态
kubectl get nodes

# 检查存储类
kubectl get storageclass
```

## 安装 Operator

### 方法一：使用部署脚本（推荐）

```bash
# 克隆项目
git clone https://github.com/robustmq/robustmq.git
cd robustmq/operator

# 使用默认配置部署
./deploy.sh

# 或指定自定义镜像
IMAGE=your-registry/robustmq-operator:v1.0.0 ./deploy.sh
```

### 方法二：手动部署

```bash
# 创建命名空间
kubectl create namespace robustmq-system

# 部署 Operator
kubectl apply -f robustmq.yaml

# 等待 Operator 就绪
kubectl wait --for=condition=available --timeout=300s \
  deployment/robustmq-operator-controller-manager -n robustmq-system
```

### 验证安装

```bash
# 检查 Operator 状态
kubectl get pods -n robustmq-system

# 检查 CRD
kubectl get crd | grep robustmq

# 查看 Operator 日志
kubectl logs -n robustmq-system deployment/robustmq-operator-controller-manager -f
```

## 部署 RobustMQ 实例

### AllInOne 部署（开发测试）

AllInOne 模式将所有服务（meta、broker、journal）运行在单个 StatefulSet 中，适合开发和测试环境。

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-dev
  namespace: default
spec:
  # 部署模式
  deploymentMode: AllInOne
  
  # 镜像配置
  image:
    repository: robustmq/robustmq
    tag: latest
    pullPolicy: IfNotPresent
  
  # AllInOne 配置
  allInOne:
    replicas: 1
    resources:
      requests:
        memory: "1Gi"
        cpu: "500m"
      limits:
        memory: "2Gi"
        cpu: "1000m"
  
  # 存储配置
  storage:
    storageClass: standard
    size: "10Gi"
    accessModes:
      - ReadWriteOnce
  
  # 网络配置
  network:
    mqtt:
      tcpPort: 1883
      tlsPort: 1885
      webSocketPort: 8083
      webSocketTLSPort: 8085
      serviceType: ClusterIP
  
  # 监控配置
  monitoring:
    enabled: true
    prometheus:
      enable: true
      port: 9091
    serviceMonitor:
      enabled: true
      interval: "30s"
```

### Microservices 部署（生产环境）

Microservices 模式将每个服务运行在独立的 StatefulSet 中，提供更好的可扩展性和隔离性。

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-prod
  namespace: default
spec:
  # 微服务部署模式
  deploymentMode: Microservices
  
  # 镜像配置
  image:
    repository: robustmq/robustmq
    tag: latest
    pullPolicy: IfNotPresent
  
  # 微服务配置
  microservices:
    metaService:
      replicas: 3
      resources:
        requests:
          memory: "512Mi"
          cpu: "250m"
        limits:
          memory: "1Gi"
          cpu: "500m"
    brokerService:
      replicas: 2
      resources:
        requests:
          memory: "1Gi"
          cpu: "500m"
        limits:
          memory: "2Gi"
          cpu: "1000m"
    journalService:
      replicas: 3
      resources:
        requests:
          memory: "1Gi"
          cpu: "500m"
        limits:
          memory: "2Gi"
          cpu: "1000m"
      storage:
        size: "50Gi"
  
  # 存储配置
  storage:
    storageClass: fast-ssd
    size: "20Gi"
    accessModes:
      - ReadWriteOnce
  
  # 网络配置
  network:
    mqtt:
      serviceType: LoadBalancer
    kafka:
      serviceType: LoadBalancer
    grpc:
      serviceType: ClusterIP
  
  # 监控配置
  monitoring:
    enabled: true
    serviceMonitor:
      enabled: true
      labels:
        prometheus: "kube-prometheus"
  
  # 安全配置
  security:
    tls:
      enabled: true
      secretName: "robustmq-tls"
      ca:
        autoGenerate: true
    auth:
      defaultUser: "admin"
      passwordSecret: "robustmq-auth"
```

### 部署实例

```bash
# 部署 AllInOne 实例
kubectl apply -f sample-robustmq.yaml

# 或部署自定义配置
kubectl apply -f your-robustmq-config.yaml

# 检查部署状态
kubectl get robustmq

# 查看详细信息
kubectl describe robustmq robustmq-sample
```

## 配置说明

### 部署模式

| 模式 | 描述 | 适用场景 |
|------|------|----------|
| AllInOne | 所有服务运行在单个 StatefulSet 中 | 开发、测试、小规模部署 |
| Microservices | 每个服务运行在独立的 StatefulSet 中 | 生产环境、大规模部署 |

### 镜像配置

```yaml
image:
  repository: robustmq/robustmq  # 镜像仓库
  tag: latest                    # 镜像标签
  pullPolicy: IfNotPresent       # 拉取策略
  pullSecrets:                   # 拉取密钥
    - name: my-registry-secret
```

### 存储配置

```yaml
storage:
  storageClass: fast-ssd         # 存储类
  size: "20Gi"                   # 存储大小
  accessModes:                   # 访问模式
    - ReadWriteOnce
```

### 网络配置

```yaml
network:
  mqtt:
    tcpPort: 1883                # MQTT TCP 端口
    tlsPort: 1885                # MQTT TLS 端口
    webSocketPort: 8083          # WebSocket 端口
    webSocketTLSPort: 8085       # WebSocket TLS 端口
    serviceType: LoadBalancer    # 服务类型
  kafka:
    port: 9092                   # Kafka 端口
    serviceType: LoadBalancer
  grpc:
    port: 1228                   # gRPC 端口
    serviceType: ClusterIP
  amqp:
    port: 5672                   # AMQP 端口
    serviceType: ClusterIP
```

### 监控配置

```yaml
monitoring:
  enabled: true                  # 启用监控
  prometheus:
    enable: true                 # 启用 Prometheus 指标
    port: 9091                   # 指标端口
    model: "pull"                # 拉取模式
  serviceMonitor:
    enabled: true                # 启用 ServiceMonitor
    interval: "30s"              # 抓取间隔
    labels:                      # 额外标签
      prometheus: "kube-prometheus"
```

### 安全配置

```yaml
security:
  tls:
    enabled: true                # 启用 TLS
    secretName: "robustmq-tls"   # TLS 密钥名称
    ca:
      autoGenerate: true         # 自动生成 CA
  auth:
    defaultUser: "admin"         # 默认用户名
    passwordSecret: "robustmq-auth"  # 密码密钥
    storageType: "placement"     # 存储类型
```

## 管理操作

### 查看状态

```bash
# 查看所有 RobustMQ 实例
kubectl get robustmq

# 查看特定实例状态
kubectl get robustmq robustmq-sample -o yaml

# 查看 Pod 状态
kubectl get pods -l app=robustmq

# 查看服务状态
kubectl get services -l app=robustmq
```

### 扩缩容

```bash
# 编辑配置进行扩缩容
kubectl edit robustmq robustmq-sample

# 或使用 patch 命令
kubectl patch robustmq robustmq-sample --type='merge' \
  -p='{"spec":{"allInOne":{"replicas":3}}}'
```

### 更新配置

```bash
# 更新镜像版本
kubectl patch robustmq robustmq-sample --type='merge' \
  -p='{"spec":{"image":{"tag":"v1.1.0"}}}'

# 更新资源配置
kubectl patch robustmq robustmq-sample --type='merge' \
  -p='{"spec":{"allInOne":{"resources":{"limits":{"memory":"4Gi"}}}}}'
```

### 删除实例

```bash
# 删除特定实例
kubectl delete robustmq robustmq-sample

# 删除所有实例
kubectl delete robustmq --all
```

## 监控和日志

### 查看日志

```bash
# 查看 Operator 日志
kubectl logs -n robustmq-system deployment/robustmq-operator-controller-manager -f

# 查看 RobustMQ Pod 日志
kubectl logs -l app=robustmq -f

# 查看特定 Pod 日志
kubectl logs robustmq-sample-0 -f
```

### 监控指标

```bash
# 检查 ServiceMonitor
kubectl get servicemonitor

# 检查 Prometheus 指标端点
kubectl port-forward svc/robustmq-sample-mqtt 9091:9091
curl http://localhost:9091/metrics
```

### 健康检查

```bash
# 检查 Pod 健康状态
kubectl get pods -l app=robustmq

# 检查服务端点
kubectl get endpoints -l app=robustmq

# 检查事件
kubectl get events --sort-by=.metadata.creationTimestamp
```

## 故障排除

### 常见问题

1. **Operator 无法启动**

   ```bash
   # 检查 RBAC 权限
   kubectl auth can-i create statefulsets --as=system:serviceaccount:robustmq-system:robustmq-operator-controller-manager
   
   # 检查 Operator 日志
   kubectl logs -n robustmq-system deployment/robustmq-operator-controller-manager
   ```

2. **Pod 启动失败**

   ```bash
   # 检查资源限制
   kubectl describe pod robustmq-sample-0
   
   # 检查存储配置
   kubectl get pvc
   
   # 检查镜像拉取
   kubectl describe pod robustmq-sample-0 | grep -A 5 "Events:"
   ```

3. **服务无法访问**

   ```bash
   # 检查服务配置
   kubectl get svc -l app=robustmq
   
   # 检查网络策略
   kubectl get networkpolicy
   
   # 测试服务连接
   kubectl run test-pod --image=busybox --rm -it -- nslookup robustmq-sample-mqtt
   ```

### 调试命令

```bash
# 进入 Pod 调试
kubectl exec -it robustmq-sample-0 -- /bin/bash

# 检查配置
kubectl exec robustmq-sample-0 -- cat /robustmq/config/server.toml

# 检查集群状态
kubectl exec robustmq-sample-0 -- ./libs/cli-command cluster info

# 检查资源使用
kubectl top pods -l app=robustmq
```

## 卸载 Operator

### 使用卸载脚本

```bash
# 使用默认配置卸载
./undeploy.sh

# 或指定命名空间
NAMESPACE=robustmq-system ./undeploy.sh
```

### 手动卸载

```bash
# 删除所有 RobustMQ 实例
kubectl delete robustmq --all --all-namespaces

# 删除 Operator
kubectl delete -f robustmq.yaml

# 删除命名空间（可选）
kubectl delete namespace robustmq-system
```

## 最佳实践

### 生产环境建议

1. **资源规划**
   - 为每个组件设置合适的资源请求和限制
   - 使用 QoS 类为关键组件提供资源保障
   - 监控资源使用情况并定期调整

2. **存储配置**
   - 使用高性能存储类（如 SSD）
   - 为不同服务配置独立的存储卷
   - 定期备份重要数据

3. **网络配置**
   - 使用 LoadBalancer 或 Ingress 暴露服务
   - 配置网络策略限制访问
   - 启用 TLS 加密

4. **监控告警**
   - 配置 Prometheus 监控
   - 设置关键指标告警
   - 定期检查集群健康状态

5. **安全配置**
   - 启用 TLS 加密
   - 配置身份验证
   - 使用 RBAC 控制访问权限

### 开发环境建议

1. **使用 AllInOne 模式**简化部署
2. **设置较小的资源限制**节省成本
3. **使用 ClusterIP 服务类型**减少配置复杂度
4. **启用详细日志**便于调试
