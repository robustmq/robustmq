# 动态配置机制

除了 topic 级配置([Topic 配置](./TopicConfig.md)),RobustMQ 还有一类**集群动态配置**,通过 admin HTTP API 设置并存储在 Meta Service 中。配置的运行时生效方式和 Broker 重启后的恢复行为取决于 `config_type`;仅仅持久化到 Meta,并不意味着每个值都会被每个 Broker 运行时组件重新加载。它不写在 `config/server.toml` 里(那是[静态配置](./BrokerConfig.md))。

## 工作原理

1. 通过 admin HTTP 接口写入一段配置,指定 `config_type`(Kafka 集群开关用 `KafkaDynamic`)。
2. 配置内容持久化到 Meta Service。
3. 配置更新应用到 Broker 的内存配置缓存 `NodeCacheManager`,并通过缓存更新传播到其他 Broker。
4. Broker 启动时,`build_cluster_config()` 会从 Meta 加载 `MqttProtocol`、`MqttSlowSubscribeConfig`、`MqttFlappingDetect`、`MqttOfflineMessage`、`MqttSchema`、`MqttSystemMonitor` 和 `KafkaDynamic`。当前 Broker 启动加载流程不会加载 `MqttLimit`、`ClusterLimit` 和 `MetaRuntime`,因此这些类型持久化的值不会在重启后恢复到 Broker 的 `NodeCacheManager` 配置中。这些类型的运行时效果也取决于使用方是否会动态读取更新后的配置。

## 以 `auto.create.topics.enable` 为例

这是目前 Kafka 集群动态配置中最典型的一项:**当客户端请求一个不存在的 topic 时,是否自动创建它**。

| 项 | 值 |
|---|---|
| 默认值 | `true`(与 Kafka broker 默认一致) |
| 是否可热改 | 是 |
| 持久化与 Broker 重启 | 存储在 Meta,并由 Broker 在启动时重新加载 |

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
| 运行时生效 | 启动时读取静态配置 | 取决于 `config_type` 及对应运行时组件 | 是 |
| 入口 | 文件 | admin HTTP | Kafka 协议 |

相关文档:[Broker 配置](./BrokerConfig.md) · [Topic 配置](./TopicConfig.md)。
