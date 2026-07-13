# Topic 配置

Topic 级配置是 **动态配置**:通过标准 Kafka 协议读写,持久化在 meta 层,不需要重启。与 [Broker 运行时配置](./BrokerConfig.md)(静态、进程级)不同,它作用于单个 topic。

## 读写方式

| 操作 | Kafka API | CLI 示例 |
|---|---|---|
| 查询配置 | `DescribeConfigs` | `kafka-configs.sh --describe --entity-type topics --entity-name <topic>` |
| 全量修改 | `AlterConfigs` | `kafka-configs.sh --alter --entity-type topics --entity-name <topic> --add-config k=v` |
| 增量修改 | `IncrementalAlterConfigs` | 同上(新版 `kafka-configs.sh` 默认走增量) |
| 建表时设定 | `CreateTopics` 的 `--config` | `kafka-topics.sh --create --topic <topic> --config k=v` |

建 topic 时通过 `--config` 传入的键会一并持久化,之后可被 `--describe` 正确回显。

## 支持的标准配置键

以下标准 Kafka topic 配置键可被存取并正确回显:

| 配置键 | 说明 |
|---|---|
| `retention.ms` | 消息保留时长 |
| `cleanup.policy` | 清理策略(`delete` / `compact`) |
| `compression.type` | 压缩类型 |
| `max.message.bytes` | 单批消息大小上限 |
| `retention.bytes` | 分区保留字节数上限 |
| `segment.bytes` | 段大小 |
| 其它标准键 | 可存储并回显 |

## 现状与限制

::: warning 当前实现状态
Topic 配置目前的行为是 **"可存取、可回显、动态源标记正确"**,但 **多数配置项尚未全部接入引擎行为**:

- 配置写入后能被持久化,`DescribeConfigs` 会以正确的动态来源标记(dynamic topic config)回显。
- 但配置值当前**不一定改变引擎的实际行为**。例如 `retention.ms` 仅部分应用;`cleanup.policy=compact`、`compression.type` 等大多尚未在存储引擎中强制生效。
- 因此可以把 topic 配置视作"元数据先行":接口与回显已就绪,行为强制生效正在逐步补齐。

配置强制生效已列入 [路线图](../Roadmap.md)。
:::

## 与 Broker 配置的关系

`max.message.bytes` 既有 broker 级默认(`kafka_runtime.max_message_bytes`,见 [Broker 配置](./BrokerConfig.md)),也可在 topic 级覆盖。集群级开关(如 `auto.create.topics.enable`)则属于另一类,见 [动态配置机制](./DynamicConfig.md)。
