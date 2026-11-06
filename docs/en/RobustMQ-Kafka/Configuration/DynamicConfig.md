# Dynamic Configuration

Beyond topic-level config ([Topic Configuration](./TopicConfig.md)), RobustMQ has **cluster dynamic config** entries stored in the Meta Service and exposed through the admin HTTP API. Runtime application and restoration after a Broker restart depend on the `config_type`; persistence in Meta alone does not mean every value is reloaded into every Broker runtime component. These entries do not live in `config/server.toml` (that is [static config](./BrokerConfig.md)).

## How it works

1. Write a config blob through the admin HTTP API, specifying a `config_type` (`KafkaDynamic` for Kafka cluster switches).
2. The config blob is persisted in Meta Service storage.
3. The update is applied to the Broker's in-memory `NodeCacheManager` configuration and propagated to other Brokers through cache updates.
4. On Broker startup, `build_cluster_config()` reloads `MqttProtocol`, `MqttSlowSubscribeConfig`, `MqttFlappingDetect`, `MqttOfflineMessage`, `MqttSchema`, `MqttSystemMonitor`, and `KafkaDynamic` from Meta. The accepted types `MqttLimit`, `ClusterLimit`, and `MetaRuntime` are not currently loaded by this Broker startup path, so their persisted values are not restored into the Broker's `NodeCacheManager` configuration after restart. Runtime effects for these types also depend on whether the consuming subsystem reads updated configuration dynamically.

## Example: `auto.create.topics.enable`

This is currently the most representative Kafka cluster dynamic config item: **whether to auto-create a topic when a client requests one that does not exist**.

| Item | Value |
|---|---|
| Default | `true` (matches Kafka's broker default) |
| Hot-changeable | Yes |
| Persistence and Broker restart | Stored in Meta and reloaded by the Broker at startup |

### Decision logic

Auto-creation happens only when **both conditions hold**:

```text
client request carries allow_auto_topic_creation = true
        AND
cluster switch auto.create.topics.enable = true
```

That is: the client (e.g. a `Metadata` request) explicitly allows auto-creation, **and** the cluster switch is on. If either is false, no topic is auto-created.

### Set via admin HTTP

```bash
curl -X POST http://<admin-host>:<http_port>/api/cluster/config/set \
  -H 'Content-Type: application/json' \
  -d '{
        "config_type": "KafkaDynamic",
        "config": "{\"auto_create_topics_enable\": false}"
      }'
```

- `config_type` is fixed to `KafkaDynamic`.
- `config` is a JSON string (mind the escaping); the field is `auto_create_topics_enable`.

::: tip Once overridden, the default no longer applies
`true` is the fallback default used when the value has **never been set explicitly**. Once you set a value through the API (whether `true` or `false`), it is persisted and the built-in default logic no longer participates — only the persisted value counts from then on, until you change it again.
:::

## Difference from other config

| Aspect | Broker static config | Cluster dynamic config | Topic config |
|---|---|---|---|
| Scope | Single node process | Whole cluster | A single topic |
| Storage | `config/server.toml` | Meta layer | Meta layer |
| Runtime effect | Static config is read at startup | Depends on `config_type` and its runtime consumers | Yes |
| Entry point | File | admin HTTP | Kafka protocol |

Related: [Broker Configuration](./BrokerConfig.md) · [Topic Configuration](./TopicConfig.md).
