# Dynamic Configuration

Beyond topic-level config ([Topic Configuration](./TopicConfig.md)), RobustMQ has a category of **cluster dynamic config**: it applies to the whole cluster, is persisted in the meta layer, can be changed online, and survives restarts. It does not live in `config/server.toml` (that is [static config](./BrokerConfig.md)).

## How it works

1. Write a config blob through the admin HTTP API, specifying a `config_type` (`KafkaDynamic` for Kafka cluster switches).
2. The config is persisted into the meta layer (Raft-backed), guaranteeing it survives restarts.
3. The change is broadcast to each node's in-memory cache (`node_cache`); subsequent requests read the cache without querying meta again.

## Example: `auto.create.topics.enable`

This is currently the most representative Kafka cluster dynamic config item: **whether to auto-create a topic when a client requests one that does not exist**.

| Item | Value |
|---|---|
| Default | `true` (matches Kafka's broker default) |
| Hot-changeable | Yes |
| Persistence | Meta layer, survives restarts |

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
| Hot-change | No (restart) | Yes | Yes |
| Entry point | File | admin HTTP | Kafka protocol |

Related: [Broker Configuration](./BrokerConfig.md) · [Topic Configuration](./TopicConfig.md).
