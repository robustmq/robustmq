# Pluggable Storage Architecture

RobustMQ decouples the Broker from its storage backend through the `StorageAdapter` trait. The Broker calls only the trait interface and has no awareness of whether the underlying implementation is Memory, RocksDB, File Segment, or an external storage system.

---

## StorageAdapter Trait

```rust
#[async_trait]
pub trait StorageAdapter {
    async fn create_shard(&self, shard: ShardConfig) -> Result<(), StorageError>;
    async fn delete_shard(&self, shard_name: String) -> Result<(), StorageError>;
    async fn list_shard(&self, filter: ShardFilter) -> Result<Vec<ShardInfo>, StorageError>;

    async fn write(&self, shard_name: String, record: Record) -> Result<u64, StorageError>;
    async fn batch_write(&self, shard_name: String, records: Vec<Record>) -> Result<Vec<u64>, StorageError>;

    async fn read_by_offset(&self, shard_name: String, offset: u64, limit: u64) -> Result<Vec<Record>, StorageError>;
    async fn read_by_tag(&self, shard_name: String, tag: String, offset: u64, limit: u64) -> Result<Vec<Record>, StorageError>;
    async fn read_by_key(&self, shard_name: String, key: String) -> Result<Option<Record>, StorageError>;

    async fn delete_by_offset(&self, shard_name: String, offset: u64) -> Result<(), StorageError>;
    async fn delete_by_key(&self, shard_name: String, key: String) -> Result<(), StorageError>;

    async fn get_offset_by_timestamp(&self, shard_name: String, timestamp: u64) -> Result<Option<u64>, StorageError>;
    async fn get_offset_by_group(&self, group_name: String, shard_name: String) -> Result<Option<u64>, StorageError>;
    async fn commit_offset(&self, group_name: String, shard_name: String, offset: u64) -> Result<(), StorageError>;
}
```

---

## Three Built-in Implementations

![Pluggable Storage Architecture](../../images/arch_pluggable_storage.png)

| Implementation | Struct | Description |
|----------------|--------|-------------|
| Memory storage | `MemoryStorageEngine` | DashMap; data is lost on process restart |
| RocksDB storage | `RocksDBStorageEngine` | Local KV store; single-node persistence |
| File Segment storage | `EngineStorageAdapter` → `StorageEngineHandler` | Segmented log; multi-replica cluster storage |

---

## Routing Mechanism

`StorageDriverManager` selects the implementation based on the `storage_type` configured on the Topic:

![Driver Routing](../../images/arch_pluggable_driver.png)

Routing logic: when the Broker writes a message, it first reads the target Topic's `storage_type` field from `BrokerCacheManager`, then looks up an already-initialized Driver instance in `driver_list` (`DashMap<String, ArcStorageAdapter>`). On a cache miss, `init_driver` creates an instance by type, stores it in the map, and returns it. Driver instances are typed as `Arc<dyn StorageAdapter>` and are shared across requests without re-initialization.

Different Topics within the same cluster can use different storage backends (mixed storage). Switching the storage type for a Topic requires only a configuration change; no Broker code modification is needed.

---

## Integrating External Storage

Any external storage system (e.g., MinIO, S3, TiKV, MySQL) can be integrated by implementing the `StorageAdapter` trait:

1. Create a new struct that implements all methods of `StorageAdapter`
2. Add the corresponding `storage_type` branch in `StorageDriverManager::init_driver`
3. Specify the new `storage_type` in the Topic configuration

---

## Shard and Segment Metadata

Shard and Segment metadata for all storage backends is managed centrally by Meta Service:

- Shard creation, deletion, and partition configuration are stored in Meta Service's metadata Raft Group
- File Segment `active_segment`, Leader information, and ISR lists are stored in Meta Service
- Brokers cache hot data in a local `StorageCacheManager` to reduce round-trips to Meta Service
