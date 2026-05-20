# RobustMQ — Code-Level Architecture Guide

This folder documents the RobustMQ codebase at the **module / file level**: what each crate
does, how the pieces fit together, and what happens under the hood at runtime.

> Audience: contributors who want to understand the kernel and add features / new protocols /
> new storage backends. All links are workspace-relative and point at real source files.

## Table of Contents

1. [Overview](01-overview.md) — what RobustMQ is, the “one binary, one storage, any protocol” model.
2. [Architecture](02-architecture.md) — 3-layer architecture, crate map, dependency graph.
3. [Startup & Runtime](03-startup-flow.md) — how `BrokerServer` boots, runtimes, role gating.
4. [Meta Service](04-meta-service.md) — Raft-based metadata kernel.
5. [Storage Layer](05-storage-layer.md) — `storage-engine` + `storage-adapter` (the unified record path).
6. [Brokers](06-brokers.md) — `broker-core`, MQTT, Kafka, NATS, AMQP, mq9.
7. [Protocol Codecs](07-protocol-codecs.md) — wire-format crate `src/protocol`.
8. [Supporting Services](08-supporting-services.md) — admin, gRPC clients, connector, rule-engine, schema-register, delay-message.
9. [Common Crates](09-common-crates.md) — `src/common/*` shared infrastructure.
10. [CLI & Binaries](10-cli-and-tooling.md) — `cmd`, `cli-command`, `cli-bench`, `bin/`.
11. [End-to-End Flows](11-end-to-end-flows.md) — publish → store → cross-protocol consume, mq9 mailbox flow.
12. [mq9-core & nats-broker](12-mq9-and-nats-broker.md) — deep dive into the NATS-compatible runtime and the mq9 agent-messaging extension.

## Reading Order

- **Quick understanding**: 1 → 2 → 3 → 11.
- **Adding a new protocol broker**: 2 → 6 → 7 → 5.
- **Adding a new storage backend**: 5 → 4 → 11.
- **Operating / observability**: 3 → 8 → 9 → 10.

## Source map (top-level)

| Path | Role |
|------|------|
| [src/broker-server](../../src/broker-server) | Single-binary launcher; composes all components |
| [src/meta-service](../../src/meta-service) | Raft metadata kernel |
| [src/storage-engine](../../src/storage-engine) | Commit log, segments, ISR replication |
| [src/storage-adapter](../../src/storage-adapter) | Trait abstraction every broker writes through |
| [src/broker-core](../../src/broker-core) | Cross-protocol state (tenants, topics, share groups, cache) |
| [src/mqtt-broker](../../src/mqtt-broker), [kafka-broker](../../src/kafka-broker), [nats-broker](../../src/nats-broker), [amqp-broker](../../src/amqp-broker), [mq9-core](../../src/mq9-core) | Protocol implementations |
| [src/protocol](../../src/protocol) | All wire-format codecs |
| [src/grpc-clients](../../src/grpc-clients) | Typed gRPC client pool |
| [src/admin-server](../../src/admin-server) | HTTP/REST admin & dashboard backend |
| [src/connector](../../src/connector), [rule-engine](../../src/rule-engine), [schema-register](../../src/schema-register), [delay-message](../../src/delay-message) | Data integration & processing |
| [src/common](../../src/common) | Shared infrastructure crates |
| [src/cmd](../../src/cmd), [cli-command](../../src/cli-command), [cli-bench](../../src/cli-bench) | CLIs and binary entrypoints |
| [bin/](../../bin) | `robust-server`, `robust-ctl`, `robust-bench` |
