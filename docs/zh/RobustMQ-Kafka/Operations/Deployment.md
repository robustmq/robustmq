# 部署

RobustMQ 是单一二进制,通过 `config/server.toml` 里的 `roles` 决定一个节点承担哪些角色。Kafka 协议由 `broker` 角色对外提供服务,数据落在 `engine`(存储)上,元数据由 `meta`(Raft)管理。

## 角色说明

| 角色 | 职责 |
|---|---|
| `meta` | 基于 Raft 的元数据服务:Controller / Coordinator、节点注册表、动态配置、位点。 |
| `broker` | 协议接入层:对外提供 Kafka(及 MQTT 等)协议服务。 |
| `engine` | File Segment 存储引擎:消息落盘、offset 索引、ISR 复制。 |

## 单节点部署

单进程同时承担三种角色,适合开发与试用:

```toml
cluster_name = "broker-server"
broker_id = 1
broker_ip = "127.0.0.1"
roles = ["meta", "broker", "engine"]
grpc_port = 1228
http_port = 58080
meta_addrs = { 1 = "127.0.0.1:1228" }

[kafka_runtime]
tcp_port = 9092
```

启动后即可用 `localhost:9092` 作为 `bootstrap.servers` 连接。

## 集群部署

多节点时,每个节点各有唯一 `broker_id`,`meta_addrs` 列出所有 meta 节点用于组成 Raft 组。关键点:

- 每个节点的 `broker_ip` 必须是集群内 / 客户端可达的地址(见下)。
- `meta_addrs` 在所有节点上保持一致,键为各 meta 节点的 `broker_id`。
- 各节点 `kafka_runtime.tcp_port` 可相同(不同机器)。

```toml
# 节点 1
broker_id = 1
broker_ip = "10.0.0.1"
roles = ["meta", "broker", "engine"]
meta_addrs = { 1 = "10.0.0.1:1228", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1228" }
```

## 关键端口

| 端口 | 配置项 | 用途 |
|---|---|---|
| 9092 | `kafka_runtime.tcp_port` | Kafka 协议接入 |
| 1228 | `grpc_port` | 节点间 gRPC / meta 通信 |
| 58080 | `http_port` | admin HTTP(含[动态配置](../Configuration/DynamicConfig.md)接口) |

## 容器 / Kubernetes 下的地址注意点

这是部署中最容易踩的坑:Kafka 客户端会按 broker **广告出来的地址**直连,而不是你 `bootstrap.servers` 里填的地址。

- `broker_ip` 决定广告地址。容器里 socket 可能 bind 在 `0.0.0.0`,但广告地址必须是**客户端可达**的值。
- Docker:若客户端在宿主机 / 其它主机,`broker_ip` 应设为宿主机可达 IP,而非容器内网 IP。
- Kubernetes:每个 broker 需要各自稳定、可路由的地址(如 headless Service + 每 Pod 地址)。
- 现象:bootstrap 成功但随后直连超时,基本都是广告地址不可达。

完整说明见 [广告地址机制](./AdvertisedListeners.md)。

## 健康检查

- **端口探活**:对 `kafka_runtime.tcp_port`(如 9092)做 TCP 探测,确认协议端口就绪。
- **admin HTTP**:`http_port`(如 58080)可用于就绪 / 存活探测与配置查询。
- **协议层验证**:用 `kafka-broker-api-versions.sh --bootstrap-server <host>:9092` 或 `kafka-topics.sh --list` 确认 Kafka 协议链路可用。
- **集群状态**:`DescribeCluster`(如 `kafka-metadata`/AdminClient)可看到 `controller_id` 与 broker 列表,`controller_id` 为 `-1` 表示 Raft Leader 尚未就绪。

相关文档:[Broker 配置](../Configuration/BrokerConfig.md) · [集群与 Controller](./ClusterAndController.md) · [广告地址机制](./AdvertisedListeners.md)。
