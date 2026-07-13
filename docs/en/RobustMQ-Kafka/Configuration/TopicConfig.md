# Topic Configuration

Topic-level configuration is **dynamic config**: read and written over the standard Kafka protocol, persisted in the meta layer, no restart needed. Unlike [Broker Runtime Configuration](./BrokerConfig.md) (static, process-level), it applies to an individual topic.

## How to read/write

| Operation | Kafka API | CLI example |
|---|---|---|
| Read | `DescribeConfigs` | `kafka-configs.sh --describe --entity-type topics --entity-name <topic>` |
| Full alter | `AlterConfigs` | `kafka-configs.sh --alter --entity-type topics --entity-name <topic> --add-config k=v` |
| Incremental alter | `IncrementalAlterConfigs` | same as above (recent `kafka-configs.sh` uses incremental by default) |
| Set on create | `CreateTopics` `--config` | `kafka-topics.sh --create --topic <topic> --config k=v` |

Keys passed via `--config` at create time are persisted too and can later be echoed back by `--describe`.

## Supported standard keys

The following standard Kafka topic config keys can be stored and echoed back correctly:

| Key | Description |
|---|---|
| `retention.ms` | Message retention time |
| `cleanup.policy` | Cleanup policy (`delete` / `compact`) |
| `compression.type` | Compression type |
| `max.message.bytes` | Max size of a single batch |
| `retention.bytes` | Max retained bytes per partition |
| `segment.bytes` | Segment size |
| Other standard keys | Stored and echoed |

## Status and limitations

::: warning Current implementation status
Topic config today behaves as **"storable, echoable, with correct dynamic-source marking"**, but **most keys are not yet fully wired into engine behavior**:

- A written config is persisted, and `DescribeConfigs` echoes it back with the correct dynamic source marking (dynamic topic config).
- However, the value does **not necessarily change actual engine behavior** yet. For example, `retention.ms` is only partially applied; `cleanup.policy=compact`, `compression.type`, etc. are mostly not enforced in the storage engine yet.
- Think of topic config as "metadata first": the API and echo are ready; behavior enforcement is being filled in incrementally.

Config enforcement is on the [Roadmap](../Roadmap.md).
:::

## Relationship to broker config

`max.message.bytes` has both a broker-level default (`kafka_runtime.max_message_bytes`, see [Broker Configuration](./BrokerConfig.md)) and a topic-level override. Cluster-level switches (e.g. `auto.create.topics.enable`) are a separate category, see [Dynamic Configuration](./DynamicConfig.md).
