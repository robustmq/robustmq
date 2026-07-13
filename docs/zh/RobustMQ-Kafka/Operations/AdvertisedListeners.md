# 广告地址机制

Kafka 客户端的连接模型是"两跳":先连 `bootstrap.servers` 里的任意 broker 拿元数据,再根据元数据里给出的地址**直连**目标 broker(分区 leader / 协调器)。因此,broker 告诉客户端的地址必须对客户端**真实可达**——这就是"广告地址"(advertised address)要解决的问题。

![广告地址机制](../../../images/kafka-advertised.svg)

## bind 地址 vs 广告地址

| 概念 | 含义 |
|---|---|
| bind 地址 | socket 实际监听的地址。容器里常为 `0.0.0.0:9092`。 |
| 广告地址 | 通过 `Metadata` / `FindCoordinator` 告诉客户端去连的 host:port。 |

两者可以不同。客户端从不使用 bind 地址,只使用广告地址。

## RobustMQ 如何注册与告知

这是一套**多协议共用的通用机制**:节点启动时,把每种协议的对外地址写入 meta 层的**节点注册表**(`NodeExtend`)。Kafka 对应其中的 `kafka.tcp_addr`(MQTT、NATS 各有自己的字段,走同一条注册 / 消费路径)。

- **广告地址 = `broker_ip` + `kafka_runtime.tcp_port`**。
- `broker_ip` 未设置时,使用本机自动探测到的 IP。
- `Metadata`(broker 列表、分区 leader)与 `FindCoordinator`(协调器)都从注册表读取 `kafka.tcp_addr`,据此告知客户端直连正确节点。

## 各环境的设置要点

| 场景 | 建议 |
|---|---|
| 本机 / 开发 | 默认 `broker_ip = "127.0.0.1"` 即可。 |
| 同网段多机集群 | 把 `broker_ip` 设为该节点在集群内可被访问的 IP。 |
| Docker | 若客户端在宿主机,`broker_ip` 需设为宿主机可达的地址(如映射出的宿主 IP),而非容器内网 IP。 |
| Kubernetes | 设为对应 Service / Pod 对客户端可达的地址;每个 broker 需要各自稳定、可路由的地址。 |
| NAT / 公网 | 设为客户端侧能访问的公网地址。 |

::: warning 常见故障
若广告地址设成了客户端不可达的地址(如容器内网 IP、`127.0.0.1` 却从远程连接),现象通常是:**bootstrap 能连上、能取到元数据,但随后直连分区 leader / 协调器超时或连接被拒**。排查时优先核对 `Metadata` 返回的 host:port 是否可从客户端 ping / telnet 通。
:::

## 相关

- 广告地址端口来自 `kafka_runtime.tcp_port`,见 [Broker 配置](../Configuration/BrokerConfig.md)。
- 协调器如何借助注册表定位,见 [集群与 Controller](./ClusterAndController.md)。
- 部署时的地址设置,见 [部署](./Deployment.md)。
