# 5. Storage Layer

Two crates make up the storage layer:

- [src/storage-adapter](../../src/storage-adapter) — **the trait** every broker uses.
- [src/storage-engine](../../src/storage-engine) — the default high-performance backend.

Other backends (MySQL, memory) live alongside as alternate driver implementations.

## 5.1 `storage-adapter` — the kernel boundary

### Files

```
src/storage-adapter/src
├── lib.rs
├── storage.rs            # the StorageAdapter trait
├── driver.rs             # StorageDriverManager (picks driver per topic/tenant)
├── engine/               # default driver delegating to storage-engine
├── mysql/                # MySQL-backed driver
├── topic.rs              # init_inner_topics – system shards
├── consumer.rs           # generic consumer state
├── consumer_priority.rs  # mq9 3-tier priority queues
├── priority.rs           # priority comparison primitives
└── tests.rs
```

### The trait

From [storage.rs](../../src/storage-adapter/src/storage.rs):

```rust
#[async_trait]
pub trait StorageAdapter {
    async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError>;
    async fn list_shard(&self, shard: Option<String>) -> Result<Vec<AdapterShardDetail>, CommonError>;
    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError>;

    async fn write(&self, shard: &str, data: &[AdapterWriteRecord])
        -> Result<Vec<AdapterWriteRespRow>, CommonError>;

    async fn read_by_offset(&self, shard: &str, offset: u64, cfg: &AdapterReadConfig)
        -> Result<Vec<StorageRecord>, CommonError>;
    async fn read_by_tag(&self, shard: &str, tag: &str, start: Option<u64>, cfg: &AdapterReadConfig)
        -> Result<Vec<StorageRecord>, CommonError>;
    async fn read_by_keys(&self, shard: &str, /* ... */) -> Result<Vec<StorageRecord>, CommonError>;
    // + offset management, deletes, watermarks
}
```

Canonical types come from [common/metadata-struct/adapter](../../src/common/metadata-struct).
The crucial invariant: **all protocols share `AdapterWriteRecord` / `StorageRecord`**.
That is what makes “MQTT publish ↔ Kafka consume” real rather than a gateway translation.

### Driver selection — `driver.rs`

`StorageDriverManager` resolves the driver to use for a given topic / tenant by reading
config (`common_config::storage::StorageType`) and instantiating one of:

- `engine` — default; talks to a local or remote `storage-engine` node.
- `memory` — in-RAM, for NATS-style low-latency / ephemeral topics.
- `rocksdb` — single-node RocksDB.
- `mysql` — external SQL store (operational metadata, low throughput topics).

The same broker can use **different drivers per topic**, configured in cluster metadata.

### Consumer & priority — `consumer*.rs`, `priority.rs`

- `consumer.rs` — generic per-group cursor management.
- `consumer_priority.rs` — implements mq9’s 3-tier (critical / urgent / normal)
  priority FETCH semantics on top of the basic shard offset model.

### System shards — `topic.rs`

`init_inner_topics` provisions internal shards used by features such as `$SYS/...`
metrics, delay-message scheduling, and mq9 control subjects.

## 5.2 `storage-engine` — the default backend

### Files

```
src/storage-engine/src
├── lib.rs
├── server/         # gRPC server exposing the engine to remote brokers
├── clients/        # inter-engine clients (replication, fetch)
├── core/           # cache, shard manager
├── commitlog/      # commit log abstraction
│   ├── memory/     # in-memory engine (NATS-style)
│   ├── rocksdb/    # RocksDB-backed engine
│   └── offset.rs   # offset bookkeeping
├── filesegment/    # append-only segment files (mmap), index files
├── handler/        # StorageEngineHandler – the API surface the adapter calls
├── isr/            # in-sync-replica replication (Kafka-style)
└── ...
```

### Concepts

- **Shard**: the unit of routing & replication. Each shard has a leader engine node and
  zero or more replicas.
- **Segment**: append-only file backing a portion of a shard’s offset space, with an
  index file for offset → file-position lookup. See `filesegment/`.
- **ISR**: the set of replicas currently in sync. `isr/` runs the replication protocol,
  acks writes once the configured quorum is replicated, and handles leader failover.
- **Commit log**: the engine abstraction supports three backends (`memory`, `rocksdb`,
  `filesegment` via `WriteManager`). Selected per topic.

### Handler — the call entry

[`handler/adapter.rs`](../../src/storage-engine/src/handler) defines `StorageEngineHandler`
with methods called by the adapter’s `engine` driver: `write`, `read_by_offset`,
`read_by_tag`, `truncate`, etc. This handler dispatches to the per-shard engine
implementation chosen at creation time.

### Why the split

- The adapter sits in the **broker process** and presents a uniform API.
- The engine can run **in-process or on a separate node**; the wire format between them
  is gRPC, so a broker-only node can transparently call a remote engine node.

## 5.3 Cold tiering & object storage

The connector + storage-engine roadmap includes automatic offload of cold segments to S3
(see [src/connector/s3](../../src/connector/src/s3)). Hot/warm/cold tiering is policy
driven through topic config (`EngineShardConfig` in metadata-struct).

Continue to [Brokers](06-brokers.md).
