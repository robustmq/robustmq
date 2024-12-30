## Prerequisites:

- [kubectl](https://kubernetes.io/docs/tasks/tools/)
- [jq](https://stedolan.github.io/jq/)
- [envsubst](https://www.gnu.org/software/gettext/manual/html_node/envsubst-Invocation.html) (`brew install gettext` on OSX)

- K8s - either:
  - [KIND](https://kind.sigs.k8s.io/docs/user/quick-start/#installation) + [Docker](https://www.docker.com) (resources: 8 CPU / 8 GRAM)
  - [Rancher Desktop](https://rancherdesktop.io) (resources: 8 CPU / 8GRAM, mobyd, and disable Traefik)

## Quickstart

步骤回顾
	1.	搭建 Kind 集群：
	•	创建并配置 Kind 集群（如端口映射、本地镜像仓库等）。
	2.	部署 Ingress-nginx:
	•	提供 HTTP 和 HTTPS 的流量路由功能，作为负载均衡器和反向代理。
	3.	部署 cert-manager:
	•	自动管理 TLS 证书，配合 Ingress 实现 HTTPS 支持。


检测k8s集群是否就绪
kubectl get pod -n ingress-nginx
kubectl get pod -n cert-manager


删除 pod
kubectl delete pod ingress-nginx-admission-create-87l75 -n ingress-nginx

查看状态
kubectl describe pod ingress-nginx-admission-create-87l75 -n ingress-nginx
kubectl describe pod ingress-nginx-admission-patch-87l75  -n ingress-nginx


docker pull k8s.gcr.io/ingress-nginx/controller:v1.1.2
docker pull k8s.gcr.io/ingress-nginx/kube-webhook-certgen:v1.1.1

docker pull quay.io/jetstack/cert-manager-cainjector:v1.6.1
docker pull quay.io/jetstack/cert-manager-controller:v1.6.1
docker pull quay.io/jetstack/cert-manager-webhook:v1.6.1

kind load docker-image k8s.gcr.io/ingress-nginx/kube-webhook-certgen:v1.1.1 --name robustmq-kind
kind load docker-image k8s.gcr.io/ingress-nginx/controller:v1.1.2 --name robustmq-kind

kind load docker-image quay.io/jetstack/cert-manager-cainjector:v1.6.1 --name robustmq-kind
kind load docker-image quay.io/jetstack/cert-manager-controller:v1.6.1 --name robustmq-kind
kind load docker-image quay.io/jetstack/cert-manager-webhook:v1.6.1 --name robustmq-kind

修正方法：动态生成唯一的 broker.id

在 StatefulSet 中，每个 Pod 的名称是唯一的，比如 kafka-0, kafka-1, kafka-2。可以通过利用这些唯一名称的特性生成唯一的 broker.id。

配置示例

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: kafka
spec:
  serviceName: kafka-service
  replicas: 3
  selector:
    matchLabels:
      app: kafka
  template:
    metadata:
      labels:
        app: kafka
    spec:
      containers:
        - name: kafka
          image: confluentinc/cp-kafka:latest
          ports:
            - containerPort: 9092
          env:
            - name: BROKER_ID
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
          command:
            - sh
            - -c
            - |
              export broker_id=${BROKER_ID##*-}
              exec kafka-server-start.sh /opt/kafka/config/server.properties --override broker.id=$broker_id
```
关键点解释
	1.	metadata.name 动态生成 Pod 名称：
每个 Pod 的名称格式为 <StatefulSet 名称>-<副本索引>，比如 kafka-0, kafka-1, kafka-2。
	2.	提取 broker.id：
使用 ${BROKER_ID##*-} 提取 Pod 名称的最后一部分（即索引），作为 broker.id。
	3.	--override 动态配置：
Kafka 启动时通过 --override broker.id=$broker_id 动态设置每个 Broker 的 ID。




6. aaa


| **特性**               | **普通 Service**                                                                                      | **Headless Service**                                                                               |
|------------------------|------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| **Service 类型**        | ClusterIP（默认）、NodePort、LoadBalancer                                                          | ClusterIP，但指定 `None`（头无主 IP）。                                                          |
| **Cluster IP**         | Service 自动分配一个 Cluster IP 地址。                                                              | 没有 Cluster IP（ClusterIP 设置为 `None`）。                                                     |
| **DNS 行为**           | 为 Service 分配一个固定的 DNS 名称，解析时返回 Cluster IP。                                           | 每个 Pod 都有一个独立的 DNS 记录，DNS 解析返回 Pod 的 IP 地址（以 Pod 的 Endpoint 为单位）。      |
| **负载均衡**           | Kubernetes 内部自动为服务后端的 Pods 提供负载均衡功能。                                              | 不提供负载均衡，客户端需要实现负载均衡（直接连接到 Pod 的 IP）。                                  |
| **适用场景**           | - 大多数常规服务，如 Web 服务或 API 服务。                                                          | - 状态服务，如数据库、ZooKeeper、Cassandra、Kafka 等。                                           |
| **Endpoint 显示行为** | Endpoint 对应的是 Service 的 Cluster IP。                                                            | Endpoint 直接对应的是后端 Pods 的 IP 和端口。                                                    |
| **配置**               | `spec.clusterIP: 自动分配`（默认值）。                                                               | `spec.clusterIP: None` 明确设置 ClusterIP 为 None。                                              |

1. 一些特殊的 Service



kubectl exec -it placement-center-0 -n robustmq -- sh

kubectl describe configmap placement-center-config  -n robustmq
