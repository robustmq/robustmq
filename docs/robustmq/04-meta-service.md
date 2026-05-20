# 4. Meta Service

Crate: [src/meta-service](../../src/meta-service)

The Meta Service is RobustMQ’s **strongly consistent metadata kernel**. It replaces what
many systems delegate to etcd or ZooKeeper. It is built on
[`openraft`](https://crates.io/crates/openraft) and persists state in RocksDB via
[common/rocksdb-engine](../../src/common/rocksdb-engine).

## Directory map

```
src/meta-service/src
├── lib.rs            # MetaServiceServerParams, entrypoint glue
├── controller/       # high-level controllers (cluster, rebalance, lifecycle)
├── core/             # in-memory caches and business logic over replicated state
├── raft/             # openraft integration
│   ├── store/        # log store + state machine (RocksDB backed)
│   ├── network/      # gRPC transport for Raft RPCs
│   ├── route/        # apply log entries → mutate state machine
│   ├── snapshot/     # snapshot install / serialize
│   ├── leadership.rs # leader-only operations
│   ├── manager.rs    # Raft node lifecycle
│   ├── group.rs      # multi-group helpers
│   ├── type_config.rs# openraft TypeConfig (NodeId, Entry, SnapshotData)
│   └── services.rs
├── server/           # gRPC service implementations
└── storage/          # typed accessors over RocksDB column families
```

## What metadata is managed

Stored as Raft state and projected into typed caches:

- **Cluster topology** — nodes, roles, addresses, heartbeats.
- **Tenants** — `metadata_struct::tenant::Tenant`.
- **Topics & shards** — for every protocol, mapped onto a shared shard namespace.
- **MQTT ACLs / users / blacklists** — used by `mqtt-broker/core/security.rs`.
- **Subscriptions & sessions** — durable MQTT state, share-group state.
- **Consumer-group offsets** — `OffsetManager` reads/writes via Meta.
- **Connectors, rules, schemas** — definitions for `connector`, `rule-engine`,
  `schema-register`.
- **mq9 AgentCards / mailbox configs** — registry data for the AI-agent layer.

## Raft layer — `raft/`

- [`type_config.rs`](../../src/meta-service/src/raft/type_config.rs) declares the
  `openraft::TypeConfig` (entry type, node ID, snapshot type).
- `store/` implements `RaftLogReader`, `RaftStorage`, and the state-machine apply path,
  all backed by RocksDB column families defined in
  [common/rocksdb-engine/storage/family.rs](../../src/common/rocksdb-engine).
- `network/` implements the `RaftNetwork` trait over gRPC, with clients coming from
  [grpc-clients/meta](../../src/grpc-clients/src/meta).
- `route/` is the **command dispatcher**: an applied Raft log entry is routed to a
  specific handler that mutates the in-memory cache (e.g. create-tenant,
  create-shard, set-acl).
- `leadership.rs` ensures writes go to the leader; followers transparently forward.

## Controllers — `controller/`

Long-running loops that observe state and drive convergence:

- Node lifecycle (mark dead, GC offline nodes).
- Shard rebalancing across engine nodes.
- Connector / rule scheduling.

Controllers run **only on the Raft leader**, gated by `leadership.rs`.

## gRPC surface — `server/`

Each typed RPC (Tenant, Topic, ACL, Cluster, Connector, Schema, Offset, ...) is exposed
through a gRPC service generated from the protos in
[src/protocol/meta](../../src/protocol/src/meta). The client side lives in
[grpc-clients/src/meta](../../src/grpc-clients/src/meta).

## How brokers see Meta

Brokers do **not** read from Meta on the hot path. Instead:

1. At boot, `start_load_cache` ([broker-server/src/load_cache.rs](../../src/broker-server/src/load_cache.rs))
   pulls a snapshot of metadata into local caches (`NodeCacheManager`, MQTT cache,
   NATS cache, etc.).
2. `start_update_cache` ([broker-server/src/update_cache.rs](../../src/broker-server/src/update_cache.rs))
   subscribes to Meta change events and keeps caches fresh.
3. Hot paths read only the local cache. Writes (create topic, set ACL) RPC into Meta,
   which then replicates and broadcasts.

This is why MQTT QoS-0 publish latency is unaffected by Raft round-trips.

Continue to [Storage Layer](05-storage-layer.md).
