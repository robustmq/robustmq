# 1. Overview

## What is RobustMQ

RobustMQ is a **unified, multi-protocol messaging engine written in Rust**. A single binary
hosts a metadata service, a storage engine, and one or more protocol brokers. All brokers
talk to the same storage abstraction, so **a message written once is consumable through any
supported protocol**.

Slogan from [README.md](../../README.md):
> *One binary, one broker, one storage layer, any protocol.*

Supported protocols (status as of this writing):

| Protocol | Crate | Status |
|----------|-------|--------|
| MQTT 3.1 / 3.1.1 / 5.0 | [src/mqtt-broker](../../src/mqtt-broker) | Production-track |
| Kafka | [src/kafka-broker](../../src/kafka-broker) | In development |
| NATS | [src/nats-broker](../../src/nats-broker) | In development |
| AMQP 0-9-1 | [src/amqp-broker](../../src/amqp-broker) | In development |
| mq9 (AI-Agent mailbox) | [src/mq9-core](../../src/mq9-core) + nats-broker | In development |

## Design Principles

1. **One kernel, many faces.** Protocols are thin parsers/routers on top of a shared
   metadata + storage kernel.
2. **No external coordinator.** Cluster consensus is via embedded Raft
   (see [src/meta-service/src/raft](../../src/meta-service/src/raft)). No etcd, no ZooKeeper.
3. **Pluggable storage tiers.** Memory, RocksDB, file segments, and (planned) S3 cold tier
   are all driven through a single `StorageAdapter` trait
   ([src/storage-adapter/src/storage.rs](../../src/storage-adapter/src/storage.rs)).
4. **Rust-native, no GC.** Predictable tail latency from edge to cloud.
5. **Single binary, role-gated.** The same `broker-server` binary can run as meta-only,
   broker-only, storage-only, or all-in-one based on configuration role flags
   (see `common_base::role` checks in [src/broker-server/src/server.rs](../../src/broker-server/src/server.rs)).

## Repository Layout (high level)

```
src/
  broker-server/       # single-binary entry: BrokerServer composes everything
  meta-service/        # Raft + metadata store
  storage-engine/      # commitlog, segments, ISR replication
  storage-adapter/     # StorageAdapter trait + driver registry
  broker-core/         # cross-protocol cache, tenants, share-groups
  mqtt-broker/  kafka-broker/  nats-broker/  amqp-broker/  mq9-core/
  protocol/            # wire codecs for every supported protocol
  grpc-clients/        # generated gRPC clients + connection pool
  admin-server/        # REST/HTTP admin & dashboard backend
  connector/           # outbound data integrations
  rule-engine/         # SQL-style data transformation
  schema-register/     # Avro / Protobuf / JSON schema validation
  delay-message/       # delayed publish
  common/              # shared crates (config, base, metrics, security, ...)
  cmd/  cli-command/  cli-bench/
tests/                 # integration test harness
```

## What “unified storage” really means

The promise *“MQTT publish → Kafka consume”* relies on three concrete things:

- A **canonical record type** in [metadata-struct/adapter/adapter_record.rs](../../src/common/metadata-struct/src/adapter)
  (`AdapterWriteRecord` / `StorageRecord`) used by every broker.
- A **single trait**, `StorageAdapter`, that every broker calls — never RocksDB / file APIs
  directly. See [src/storage-adapter/src/storage.rs](../../src/storage-adapter/src/storage.rs).
- A **shared shard namespace** managed by Meta Service. Protocols map their own concepts
  (MQTT topic, Kafka topic+partition, NATS subject) onto the same shards.

If you remember nothing else: **the `StorageAdapter` trait is the kernel boundary.**

Continue to [Architecture](02-architecture.md).
