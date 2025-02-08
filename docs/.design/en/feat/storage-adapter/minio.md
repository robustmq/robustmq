# directory and file structure

`/records/{namespace}/{shard}`: All records in shard `shard` under namespace `namespace`. (append only)

serialized_record_1
serialized_record_2
...

`/offsets/{namespace}-{shard}`: store serialized shard offset struct for the shard under namespace. The struct contains namespace, shard_name and offset

serialized_shard_offset_struct

`/shard_configs/{namespace}-{shard}`: store serialized shard config struct for the shard under namespace. The struct contains namespace, shard_name and config

serialized_shard_config_struct

`/groups/{group}`: store group offsets for each namespace + shard

serialized_shard_offset_struct
serialized_shard_offset_struct
...

# Index structure

MinIO and other S3 compatible services are extremely slow. We need to design an in-memory buffer structure and read / write from S3 when necessary

```rust
struct InMemoryBuffer {
    // (namespace-shard, offset)
    buffered_offsets: DashMap<String, u64>,
    // (namespace-shard, (key, record))
    buffered_key_index: DashMap<String, DashMap<String, Record>>,
    // (namespace-shard, (tag, records))
    buffered_tag_index: DashMap<String, DashMap<String, Vec<Record>>>,
    // (group, (namespace-shard, offset))
    buffered_groups: DashMap<String, DashMap<String, u64>>,
    // (namespace-shard, config)
    buffered_shard_config: DashMap<String, ShardConfigStore>,
}
```

## How to init this buffer structure

Populate `InMemoryBuffer` with contents from S3 during initialization. Assume we have infinite memory.

## update

Always update `InMemoryBuffer` after successfully writing into S3.

When updating shard offsets, send a write request and modify `InMemoryBuffer::buffered_offsets`.

Similar when updating group offsets and shard config.

When writing new records, send write requests using a OpenDAL `Writer` with append mode + concurrent mode (8MB per API call) and update indices in `InMemoryBuffer::buffered_key_index` and `InMemoryBuffer::buffered_tag_index`:

```rust
let mut record_writer = op
    .writer_with(&Self::records_path(&namespace, &shard_name))
    .append(true)
    .chunk(8 * 1024 * 1024)
    .concurrent(8)
    .await?;
```

All read operations directly read from `InMemoryBuffer`

`
