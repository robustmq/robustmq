# Storage Adapter Architecture

Storage Adapter is RobustMQ's storage abstraction layer, sitting between the Broker (MQTT, Kafka, and other protocol handlers) and the Storage Engine (underlying storage backends). Its core responsibility is: **unifying different protocol storage concepts into the Shard abstraction, and transparently routing read/write operations to the appropriate storage backend — fully decoupling the upper-layer protocols from the lower-layer storage.**

---

## Overall Structure

```
MQTT Broker / Kafka Broker / ...
           │ Topic name
           ▼
  StorageDriverManager          ← Entry point, routes by Topic to the appropriate Driver
           │ lookup Topic config
           ├── BrokerCacheManager (Topic metadata cache)
           │
           ▼
  EngineStorageAdapter           ← Implements the StorageAdapter trait
           │
           ▼
  StorageEngineHandler           ← Unified storage engine handler
     ┌─────┼──────────┐
     │     │          │
  Memory  RocksDB  FileSegment   ← Three storage backends
```

---

## Core Concept: Shard

Shard is the core abstraction unit of Storage Adapter, unifying storage concepts across different protocols:

| Protocol | Original Concept | Maps To |
|----------|-----------------|---------|
| MQTT | Topic | Shard |
| Kafka | Partition | Shard |
| AMQP (planned) | Queue | Shard |

One Topic corresponds to a group of Shards (multi-partition). Shard naming convention:

```
{topic_id}_{partition_seq}
```

Each Shard has its own storage type configuration (`EngineMemory` / `EngineRocksDB` / `EngineSegment`). Shard and Segment metadata are managed centrally by Meta Service.

---

## Key Components

### StorageAdapter Trait

The unified interface that all storage backends must implement, defining all storage operations available to Brokers:

| Method | Description |
|--------|-------------|
| `create_shard` | Create a Shard (corresponds to creating a Topic / Partition) |
| `delete_shard` | Delete a Shard |
| `list_shard` | List Shards |
| `write` | Write a single message, return Offset |
| `batch_write` | Batch write messages |
| `read_by_offset` | Read messages by Offset |
| `read_by_tag` | Read messages by Tag |
| `read_by_key` | Read messages by Key |
| `delete_by_key` | Delete message by Key |
| `delete_by_offset` | Delete message by Offset |
| `get_offset_by_timestamp` | Find Offset corresponding to a timestamp |
| `get_offset_by_group` | Query consumer group Offset |
| `commit_offset` | Commit consumer group Offset |

### StorageDriverManager

The externally-facing entry point for Storage Adapter. Brokers call this component directly.

| Field | Type | Description |
|-------|------|-------------|
| `driver_list` | `DashMap<String, ArcStorageAdapter>` | Driver cache keyed by storage type — avoids repeated initialization |
| `engine_storage_handler` | `Arc<StorageEngineHandler>` | Underlying engine handler |
| `broker_cache` | `Arc<BrokerCacheManager>` | Topic metadata cache, queried during routing |
| `offset_manager` | `Arc<OffsetManager>` | Consumer group Offset management |
| `message_seq` | `AtomicU64` | Global write sequence counter for round-robin partition selection |

**Topic routing logic (`build_driver`):**

1. Query Topic config (including `storage_type`) from `BrokerCacheManager`
2. Look up the `driver_list` cache by `storage_type` — return directly on cache hit
3. On cache miss: initialize the Driver and insert into cache

### EngineStorageAdapter

Implements the `StorageAdapter` trait by delegating calls to `StorageEngineHandler`. It is the bridge between Storage Adapter and Storage Engine.

---

## Write Flow

```
Broker::publish(topic, message)
        │
        ▼
StorageDriverManager::write(topic_name, records)
        │
        ├── 1. Lookup Topic config (BrokerCacheManager)
        ├── 2. Select target partition: message_seq % topic.partition (round-robin)
        ├── 3. Build shard_name = build_storage_name(topic_id, partition)
        │
        ▼
EngineStorageAdapter::batch_write(shard_name, records)
        │
        ▼
StorageEngineHandler::batch_write(shard_name, records)
        │
        ├── Query StorageCacheManager for Shard config and active_segment
        ├── Check if current node is the Leader of active_segment
        │
        ├── Is Leader: write locally
        │       ├── EngineMemory → MemoryStorageEngine (DashMap in-memory)
        │       ├── EngineRocksDB → RocksDBStorageEngine (RocksDB Column Family)
        │       └── EngineSegment → WriteManager (segmented file log)
        │
        └── Not Leader: forward to Leader node
                └── ClientConnectionManager::write_send(leader_broker_id, ...)
```

### Partition Selection Strategy

Round-robin using a global atomic counter:

```
partition = message_seq.fetch_add(1) % topic.partition
```

---

## Topic Creation Flow

Topic creation is a coordinated process across Meta Service and Storage Engine:

```
1. Broker calls create_topic_full(topic, shard_config)
        │
        ▼
2. Register Topic to Meta Service (gRPC: placement_create_topic)
        │
        ▼
3. Wait for Topic to appear in BrokerCacheManager (timeout: 30s)
   Meta Service → push config → Broker local cache updated
        │
        ▼
4. StorageDriverManager::create_storage_resource(topic_name)
        │
        ▼
5. Register Shard to Meta Service (gRPC: create_shard)
        │
        ▼
6. Wait for Shard + Segment to appear in StorageCacheManager (timeout: 3s)
```

---

## Three Storage Backends

### EngineMemory (In-Memory)

Pure in-memory storage based on `DashMap` with multi-dimensional indexes:

| Index | Structure | Purpose |
|-------|-----------|---------|
| Primary data | `DashMap<shard, DashMap<offset, Record>>` | Read by Offset |
| Tag index | `DashMap<shard, DashMap<tag, Vec<offset>>>` | Query by Tag |
| Key index | `DashMap<shard, DashMap<key, offset>>` | Query by Key (unique) |
| Timestamp index | `DashMap<shard, DashMap<timestamp, offset>>` | Find Offset by time |

**Characteristics**: Microsecond latency. Data lost on process restart. Suitable for real-time metrics, ephemeral notifications.

### EngineRocksDB (RocksDB Persistent)

Persistent storage backed by RocksDB, using a dedicated Column Family (`DB_COLUMN_FAMILY_BROKER`). Per-Shard write locks prevent concurrent write conflicts.

**Characteristics**: Millisecond latency, durable storage, supports read by Offset / Tag / Key. Suitable for IoT device messages, offline messages.

### EngineSegment (Segmented File Log)

A segmented log file storage engine managed by `WriteManager`:

- Each Shard consists of multiple Segment files (rolling log)
- Each Segment has a designated Leader Broker node responsible for writes
- Segment metadata (start/end Offset, status, etc.) is managed by Meta Service
- Writes are processed in batches via async channels (`WriteManager`) for high throughput
- Supports Tag and Key index construction (`save_index`)

**Characteristics**: High-throughput durable storage with Segment rolling. Suitable for big data streams, Kafka scenarios.

---

## Offset Management

`OffsetManager` handles consumer group Offset management with two storage strategies:

| Strategy | Implementation | Description |
|----------|---------------|-------------|
| Cache storage (`enable_cache=true`) | `OffsetCacheManager` | Local RocksDB cache, low latency |
| Persistent storage | `OffsetStorageManager` | Writes to Meta Service, strong consistency |

Offset queries support timestamp-based lookup strategies (`AdapterOffsetStrategy`) to find the Offset at a specific point in time.

---

## Design Highlights

| Feature | Description |
|---------|-------------|
| **Protocol-transparent** | Brokers only pass Topic name; storage type is determined by Topic config, fully transparent to Broker |
| **Topic-level storage config** | Different Topics can use different backends; mixed storage in a single cluster is supported |
| **Leader-aware forwarding** | Write operations automatically identify the Segment Leader; non-Leader nodes transparently forward, Broker needs no topology awareness |
| **Lazy Driver initialization** | Drivers are initialized on demand and cached, avoiding resource contention at startup |
| **Unified Offset semantics** | All three backends follow the same Offset model, ensuring consistent consumer logic |
