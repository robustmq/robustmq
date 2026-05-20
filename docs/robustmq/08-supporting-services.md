# 8. Supporting Services

These crates extend the kernel without sitting on the hot path.

## 8.1 `admin-server` — HTTP / Dashboard backend

Crate: [src/admin-server](../../src/admin-server)

```
src/admin-server/src
├── lib.rs
├── server.rs       # axum-based HTTP server
├── state.rs        # HttpState: shared handles (caches, managers, pprof guard)
├── path.rs         # route table
├── client.rs       # outbound calls to other components
├── debug.rs        # pprof + debug endpoints
├── cluster/        # cluster topology APIs
├── engine/         # storage-engine introspection APIs
├── mqtt/           # MQTT-specific admin endpoints (sessions, retain, ACL)
├── mq9/            # mq9 admin endpoints (agents, mailboxes)
├── mcp/            # Model Context Protocol facade
└── tool/           # operator tools (export, import, repair)
```

Started by `BrokerServer::start_admin_server` ([server.rs](../../src/broker-server/src/server.rs)).
`HttpState` is built from the same managers used by brokers, so admin endpoints reflect
live runtime state with zero extra round-trips.

The dashboard frontend lives outside this repo and consumes these REST APIs.

## 8.2 `grpc-clients` — typed gRPC pool

Crate: [src/grpc-clients](../../src/grpc-clients)

```
src/grpc-clients/src
├── pool.rs   # ClientPool: per-address channel pool with health checks
├── macros.rs # boilerplate-removing macros for client methods
├── utils.rs
├── meta/     # clients for every Meta gRPC service
└── broker/   # clients for broker ↔ broker / engine RPCs
```

`ClientPool` is constructed once in `BrokerServer::init_base` and threaded into every
component that needs to reach a peer. The `macros.rs` file generates retry + timeout
wrappers so call sites stay clean.

## 8.3 `connector` — outbound integrations

Crate: [src/connector](../../src/connector)

A connector subscribes (via `StorageAdapter`) to one or more shards and ships records to
an external system. Each subdirectory is a self-contained driver:

```
src/connector/src
├── core.rs        # Connector lifecycle (start, stop, checkpoint)
├── loops.rs       # main poll/dispatch loop
├── manager.rs     # registration / scheduling
├── traits.rs      # ConnectorDriver trait
├── failure.rs     # retry/backoff/DLQ helpers
├── heartbeat.rs
├── storage/       # connector state persistence
├── webhook/  kafka/  mqtt_bridge/  pulsar/  rabbitmq/  redis/
├── mysql/  postgres/  mongodb/  cassandra/  clickhouse_connector/
├── influxdb_connector/  opentsdb/  greptimedb/  elasticsearch/
├── s3/   file/
```

Definitions are stored in Meta; controllers in `meta-service/controller/` schedule them
onto broker nodes. The skill at
[.claude/skills/connector-delivery](../../.claude/skills/connector-delivery) documents the
delivery conventions used when adding a new connector.

## 8.4 `rule-engine` — in-flight transformation

Crate: [src/rule-engine](../../src/rule-engine)

```
src/rule-engine/src
├── lib.rs
├── rule_trait.rs   # Rule trait (input record → output record(s))
├── decode.rs       # decode incoming records to internal value
├── encode.rs       # encode outgoing records
├── operator/       # SQL-like operators: select, where, extract, transform...
└── test_data.rs
```

Rules are SQL-style pipelines compiled from definitions stored in Meta. They run between
the broker write path and connector dispatch, enabling per-tenant routing,
field extraction (the open `operator/extract.rs` belongs here), and format conversion.

## 8.5 `schema-register` — record schemas

Crate: [src/schema-register](../../src/schema-register)

```
src/schema-register/src
├── lib.rs
├── schema.rs    # Schema entity + validation entrypoint
├── avro.rs
├── protobuf.rs
└── json.rs
```

When a topic has a registered schema, the broker validates / converts payloads via this
crate before persisting. Schema metadata is stored in Meta.

## 8.6 `delay-message` — scheduled publish

Crate: [src/delay-message](../../src/delay-message)

```
src/delay-message/src
├── lib.rs
├── manager.rs   # DelayMessageManager
├── delay.rs     # enqueue with deliver-at timestamp
├── pop.rs       # poll due messages and re-publish
└── recover.rs   # restore queue from storage on restart
```

Backs MQTT 5 delayed publish and is reusable by other protocols. Persisted via
`StorageAdapter` into a dedicated internal shard so timers survive restart.

Continue to [Common Crates](09-common-crates.md).
