# 动态配置机制

除了 topic 级配置([Topic 配置](./TopicConfig.md)),RobustMQ 还有一类 **集群动态配置**:作用于整个集群、持久化在 meta 层、可在线热改、跨重启保留。它不写在 `config/server.toml` 里(那是[静态配置](./BrokerConfig.md))。

## 工作原理

1. 通过 admin HTTP 接口写入一段配置,指定 `config_type`(Kafka 集群开关用 `KafkaDynamic`)。
2. 配置被持久化到 meta 层(基于 Raft),保证跨重启保留。
3. 变更广播到各节点的内存缓存(`node_cache`),后续请求直接读缓存,无需再查 meta。

## 以 `auto.create.topics.enable` 为例

这是目前 Kafka 集群动态配置中最典型的一项:**当客户端请求一个不存在的 topic 时,是否自动创建它**。

| 项 | 值 |
|---|---|
| 默认值 | `true`(与 Kafka broker 默认一致) |
| 是否可热改 | 是 |
| 持久化 | meta 层,跨重启保留 |

### 判定逻辑

自动创建只有在**两个条件同时成立**时才发生:

```text
客户端请求带 allow_auto_topic_creation = true
        且
集群开关 auto.create.topics.enable = true
```

即:客户端(如 `Metadata` 请求)显式允许自动建表,**并且**集群开关为开。任一为否都不会自动创建。

### 通过 admin HTTP 设置

```bash
curl -X POST http://<admin-host>:<http_port>/api/cluster/config/set \
  -H 'Content-Type: application/json' \
  -d '{
        "config_type": "KafkaDynamic",
        "config": "{\"auto_create_topics_enable\": false}"
      }'
```

- `config_type` 固定为 `KafkaDynamic`。
- `config` 是 JSON 字符串(注意转义),字段为 `auto_create_topics_enable`。

::: tip 默认值一旦被覆盖便不再生效
`true` 是"未显式设置过"时的兜底默认。一旦通过上述接口手动设过一次值(无论 `true` 还是 `false`),该值会被持久化,内置默认逻辑不再参与——之后只以持久化的值为准,直到你再次修改它。
:::

## 与其它配置的区别

| 维度 | Broker 静态配置 | 集群动态配置 | Topic 配置 |
|---|---|---|---|
| 作用域 | 单节点进程 | 整个集群 | 单个 topic |
| 存储 | `config/server.toml` | meta 层 | meta 层 |
| 热改 | 否(需重启) | 是 | 是 |
| 入口 | 文件 | admin HTTP | Kafka 协议 |

相关文档:[Broker 配置](./BrokerConfig.md) · [Topic 配置](./TopicConfig.md)。
