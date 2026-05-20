# 2. Architecture

## The Three Layers

```
┌─────────────────────────────────────────────────────────────┐
│  Broker layer  (protocol parsers, sessions, dispatch)       │
│  mqtt-broker · kafka-broker · nats-broker · amqp-broker     │
│  mq9-core                                                   │
├─────────────────────────────────────────────────────────────┤
│  Storage layer                                              │
│  storage-adapter (trait)  ──▶  storage-engine               │
│                          ╲──▶  (memory · rocksdb · file)    │
├─────────────────────────────────────────────────────────────┤
│  Meta layer  (Raft state machine)                           │
│  meta-service                                               │
└─────────────────────────────────────────────────────────────┘

         ▲ shared by every layer ▲
    common/* · protocol · grpc-clients · broker-core
```

Boundaries are strict:

- **Broker → Storage**: only through `StorageAdapter`. Brokers never touch RocksDB or files
  directly.
- **Broker / Storage → Meta**: only through typed gRPC clients in
  [src/grpc-clients](../../src/grpc-clients).
- **Meta → others**: Meta does not call brokers; it publishes change events that brokers
  pull / cache (`update_cache.rs`, `load_cache.rs` in `broker-server`).

This is why adding a new protocol only needs a new `*-broker` crate, and adding a new
storage backend only needs a new `StorageAdapter` implementation.

## Composition: the single binary

The `broker-server` crate ([src/broker-server](../../src/broker-server)) is the glue:

- [lib.rs](../../src/broker-server/src/lib.rs) — defines `BrokerServer`, which holds
  per-component **params** structs (`MetaServiceServerParams`, `MqttBrokerServerParams`,
  `KafkaBrokerServerParams`, `NatsBrokerServerParams`, `AmqpBrokerServerParams`,
  `StorageEngineParams`) plus shared resources (`ClientPool`, `RocksDBEngine`,
  `ConnectionManager`, `NodeCacheManager`, `OffsetManager`, `TaskSupervisor`,
  `GlobalRateLimiterManager`).
- [server.rs](../../src/broker-server/src/server.rs) — `start_*` methods. Each is guarded
  by role checks (`is_broker_node`, `is_engine_node`, etc.) so a node only spins up what
  it is configured to host.
- Per-component modules: [meta.rs](../../src/broker-server/src/meta.rs),
  [mqtt.rs](../../src/broker-server/src/mqtt.rs),
  [kafka.rs](../../src/broker-server/src/kafka.rs),
  [nats.rs](../../src/broker-server/src/nats.rs),
  [amqp.rs](../../src/broker-server/src/amqp.rs),
  [engine.rs](../../src/broker-server/src/engine.rs),
  [grpc.rs](../../src/broker-server/src/grpc.rs),
  [connection.rs](../../src/broker-server/src/connection.rs),
  [daemon.rs](../../src/broker-server/src/daemon.rs).

## Runtime Topology

`BrokerServer::init_base` creates **four dedicated Tokio runtimes**:

| Runtime | Purpose |
|---------|---------|
| `server-runtime` | HTTP admin server, gRPC server, lightweight control plane |
| `meta-runtime` | Raft state machine (so `openraft`-spawned tasks land here) |
| `broker-runtime` | Per-protocol broker workers (MQTT/Kafka/NATS/AMQP/mq9) |
| `engine-runtime` | Storage engine I/O and replication |

This isolates blocking patterns (e.g. RocksDB compaction, segment fsync) from latency-
sensitive paths (MQTT dispatch). Thread counts come from `BrokerConfig.runtime.*` and
helpers in `common_base::runtime`.

## Dependency Direction

```
broker-server ──▶ {meta-service, *-broker, storage-engine, admin-server,
                   storage-adapter, broker-core, common/*, grpc-clients}

*-broker      ──▶ {storage-adapter, broker-core, protocol, grpc-clients,
                   common/*}

storage-adapter ──▶ {storage-engine (default driver), common/*}

storage-engine  ──▶ {common/rocksdb-engine, common/mmap-file, grpc-clients,
                     common/*}

meta-service    ──▶ {common/rocksdb-engine, common/*, grpc-clients}

connector / rule-engine / schema-register / delay-message
                ──▶ {storage-adapter, broker-core, common/*}
```

There are **no cycles**: brokers depend on storage and meta, never the other way.

## Cluster View

A RobustMQ cluster is a set of `broker-server` processes. Each process advertises its
roles (broker / engine / meta) through Meta. Meta replicates cluster state via Raft;
brokers and engine nodes pull/refresh their caches from Meta and exchange data over gRPC
using pooled clients ([src/grpc-clients/src/pool.rs](../../src/grpc-clients/src/pool.rs)).

Continue to [Startup & Runtime](03-startup-flow.md).
