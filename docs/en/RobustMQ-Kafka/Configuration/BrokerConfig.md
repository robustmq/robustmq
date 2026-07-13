# Broker Runtime Configuration

RobustMQ Kafka configuration comes in two kinds:

- **Broker runtime (static) config**: lives in `config/server.toml`, loaded at process startup; changes require a node restart. This page covers this kind.
- **Cluster dynamic config**: persisted in the meta layer, changeable online and preserved across restarts. See [Dynamic Configuration](./DynamicConfig.md).

Kafka-related static config lives under `[kafka_runtime]` (SASL in its sub-section `[kafka_runtime.sasl]`). Every item has a default and runs on that default when unset.

## `[kafka_runtime]`

| Key | Default | Description |
|---|---|---|
| `tcp_port` | `9092` | Kafka protocol listen port. Also the port of the advertised address a node registers into meta (see [Advertised Address](../Operations/AdvertisedListeners.md)). |
| `max_fetch_bytes` | `4194304` (4 MiB) | Per-partition upper bound on how many bytes a `Fetch` response may return, regardless of the client's `max_bytes` / `partition_max_bytes`. |
| `max_message_bytes` | `1048588` (~1 MiB) | Upper bound on a single produced record batch. Larger batches are rejected with `MESSAGE_TOO_LARGE`, matching Kafka's `message.max.bytes` / topic `max.message.bytes`. |
| `max_describe_topic_partitions` | `2000` | Upper bound on how many partitions a single `DescribeTopicPartitions` response returns, regardless of the client's `response_partition_limit`. |

## `[kafka_runtime.sasl]`

| Key | Default | Description |
|---|---|---|
| `enabled` | `false` | When `false`, connections are accepted without authentication and the SASL handshake/authenticate handlers stay inert. Set `true` to enable SASL. |
| `mechanisms` | `["SCRAM-SHA-256", "SCRAM-SHA-512"]` | SASL mechanisms the broker offers, advertised to clients via `SaslHandshake`. |

## Example

```toml
[kafka_runtime]
tcp_port = 9092
max_fetch_bytes = 4194304
max_message_bytes = 1048588
max_describe_topic_partitions = 2000

[kafka_runtime.sasl]
enabled = true
mechanisms = ["SCRAM-SHA-256", "SCRAM-SHA-512"]
```

## About the advertised address

The broker address a client receives is not decided by `tcp_port` alone: it is the **`broker_ip` (top-level config) + `tcp_port`** advertised address. When `broker_ip` is unset, the auto-detected local IP is used. In Docker / Kubernetes / NAT environments, connections fail if the advertised address is unreachable from the client. See [Advertised Address](../Operations/AdvertisedListeners.md).

## Related

- Top-level `broker_ip`, `roles`, `grpc_port`, `http_port`, etc. are global node config affecting all protocols. See [Deployment](../Operations/Deployment.md).
- Topic-level keys (e.g. `retention.ms`, `cleanup.policy`) are dynamic config read/written over the Kafka protocol. See [Topic Configuration](./TopicConfig.md).
