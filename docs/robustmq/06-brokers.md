# 6. Brokers

This chapter covers `broker-core` and each protocol broker. The pattern is identical
across protocols: parse → manage sessions → dispatch → persist via `StorageAdapter`.

## 6.1 `broker-core` — cross-protocol primitives

Crate: [src/broker-core](../../src/broker-core)

```
src/broker-core/src
├── lib.rs
├── cache.rs           # NodeCacheManager – cluster topology cache
├── cluster.rs         # cluster state helpers
├── dynamic_config.rs  # hot-reloadable config knobs
├── heartbeat.rs       # node registration + heartbeat to Meta
├── inner_topic.rs     # reserved system topics
├── share_group.rs     # shared subscription group state
├── tenant.rs          # tenant lifecycle (try_init_default_tenant)
├── topic.rs           # topic helpers, naming rules
└── tool.rs
```

Every protocol broker depends on `broker-core` for cluster awareness, tenant scoping, and
shared-subscription accounting. `register_node_and_start_heartbeat` from
[heartbeat.rs](../../src/broker-core/src/heartbeat.rs) is what makes a node visible in the
Raft cluster view.

## 6.2 `mqtt-broker` — the reference protocol

Crate: [src/mqtt-broker](../../src/mqtt-broker)

```
src/mqtt-broker/src
├── broker.rs          # MqttBrokerServerParams
├── server/            # TCP/TLS/WebSocket/QUIC listeners (uses common/network-server)
├── core/              # per-feature handlers (one file ≈ one MQTT feature)
│   ├── command.rs         # CONNECT/PUBLISH/SUBSCRIBE/... dispatch
│   ├── session.rs         # session state, clean-start vs persistent
│   ├── connection.rs      # connection-level state
│   ├── keep_alive.rs      # PINGREQ/PINGRESP, idle timeout
│   ├── pkid_manager.rs    # packet-ID allocation per session
│   ├── qos.rs             # QoS 0/1/2 flows
│   ├── subscribe.rs       # SUBSCRIBE handling, retained delivery
│   ├── sub_share.rs       # shared subscriptions ($share/group/topic)
│   ├── sub_wildcards.rs   # + and # matching
│   ├── sub_auto.rs        # auto-subscription on connect
│   ├── sub_exclusive.rs   # MQTT 5 exclusive subscriptions
│   ├── sub_option.rs      # No-Local, Retain-As-Published, Retain-Handling
│   ├── sub_slow.rs        # slow-consumer detection
│   ├── retain.rs          # retained messages
│   ├── last_will.rs       # LWT publish on disconnect
│   ├── offline_message.rs # store-and-forward for persistent sessions
│   ├── delay_message.rs   # MQTT 5 delayed publish (uses delay-message crate)
│   ├── inner.rs           # internal control topics
│   ├── content_type.rs / topic_rewrite.rs / string_validator.rs
│   ├── security.rs        # authn/authz against ACL cache
│   ├── flapping_detect.rs # connect-flap detection
│   ├── limit.rs           # per-client / per-topic rate limit
│   ├── tenant.rs / topic.rs
│   ├── metrics.rs / metrics_cache.rs
│   ├── system_alarm.rs
│   ├── event.rs           # bus for internal events
│   └── cache.rs / dynamic_cache.rs / error.rs
├── subscribe/         # dispatch kernel (see below)
├── storage/           # session/retain/subscription persistence via StorageAdapter
└── system_topic/      # $SYS/... topic generation
```

### The dispatch kernel — `subscribe/`

Files: [manager.rs](../../src/mqtt-broker/src/subscribe/manager.rs),
[parse.rs](../../src/mqtt-broker/src/subscribe/parse.rs),
[push.rs](../../src/mqtt-broker/src/subscribe/push.rs),
[directly_push.rs](../../src/mqtt-broker/src/subscribe/directly_push.rs),
[share_push.rs](../../src/mqtt-broker/src/subscribe/share_push.rs),
[buckets.rs](../../src/mqtt-broker/src/subscribe/buckets.rs),
[push_model.rs](../../src/mqtt-broker/src/subscribe/push_model.rs).

- `manager.rs` keeps an in-memory subscription index (concrete + wildcard sets per
  tenant).
- `parse.rs` matches an incoming publish against the index.
- `push.rs` is the top-level fan-out, choosing between `directly_push` (single subscriber)
  and `share_push` (round-robin / sticky across a share group).
- `buckets.rs` shards work across worker tasks to keep dispatch latency stable under load.
- `push_model.rs` defines push vs pull semantics and back-pressure.

### Storage integration — `storage/`

Sessions, retained messages, subscriptions, last-will, offline queues all persist through
`StorageAdapter`. The MQTT broker never opens a RocksDB handle itself.

## 6.3 `kafka-broker`

Crate: [src/kafka-broker](../../src/kafka-broker)

```
src/kafka-broker/src
├── broker.rs       # KafkaBrokerServerParams
├── server/         # TCP listener, Kafka wire-protocol framing
├── kafka/          # request/response domain types
└── handler/        # one handler per Kafka API key (PRODUCE, FETCH, METADATA, ...)
```

Kafka topics+partitions map onto storage shards. Consumer-group state goes through
`common/group::OffsetManager` and Meta. Decoding is delegated to
[src/protocol/kafka](../../src/protocol/src/kafka).

## 6.4 `nats-broker` and `mq9-core`

Crate: [src/nats-broker](../../src/nats-broker), [src/mq9-core](../../src/mq9-core)

```
src/nats-broker/src
├── broker.rs
├── core/           # connection / subject state
├── nats/           # core NATS subjects & verbs
├── jstream/        # JetStream-style streams
├── mq9/            # mq9 subjects ($mq9.AI.AGENT.*, $mq9.AI.MAILBOX.*, $mq9.AI.MSG.*)
├── push/           # subscriber push
├── handler/        # per-verb handlers
├── server/         # TCP listener, NATS protocol framing
└── storage/        # subject → shard persistence
```

```
src/mq9-core/src
├── command.rs      # mq9 control command set
├── protocol.rs     # mq9 message envelope, AgentCard
└── lib.rs
```

mq9 is intentionally implemented **inside the NATS broker** — it reuses NATS subjects and
JetStream semantics, with a dedicated `mq9/` submodule plus the shared `mq9-core` crate
for protocol structs. Discovery uses semantic search from
[common/search-engine](../../src/common/search-engine) +
[common/llm-engine](../../src/common/llm-engine) (LanceDB + fastembed).

## 6.5 `amqp-broker`

Crate: [src/amqp-broker](../../src/amqp-broker)

```
src/amqp-broker/src
├── broker.rs
├── server/         # AMQP 0-9-1 frame listener
├── amqp/           # exchanges, queues, bindings
└── handler/        # method handlers (Connection.*, Channel.*, Basic.*)
```

Exchanges and queues map onto shards; routing keys are matched in-memory per channel.

Continue to [Protocol Codecs](07-protocol-codecs.md).
