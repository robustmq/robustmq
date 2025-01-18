# Column family (CF)

We need 3 CFs:

- `DB_COLUMN_FAMILY_OFFSET`: All offset data (type `u64`) will be stored under this CF
- `DB_COLUMN_FAMILY_RECORD`: All shard records will be stored under this CF

# design choice

We only have one copy of `Record` in rocksdb but we build multiple indices on one `Record`.

# Types of KV pairs

## in `DB_COLUMN_FAMILY_RECORD`

### shard records

key: `{namespace}_{shard}_record_{record_offset}`

value: record data

## in `DB_COLUMN_FAMILY_OFFSET`

### shard offsets

key: `{namespace}_{shard}_shard_offset`

value: **next** offset of shard `shard` under the namespace `namespace`

When a new shard is created under a namespace, a new key will be generated with value `0`.

When a shard is deleted, we will only delete the shard offset key associated with this shard. The stored record will be deleted in the background.

This key value pair will be incremented when calling `write` or `batch_write` method

### key offset

key: `{namespace}_{shard}_key_{key}_offset`

value: the offset of the record in shard `shard` under namespace `namespace` with `record.key = key`

Note: the key of a record should be unique under any `{namespace}_{shard}` pair

This key value pair is used for fast record retrieval in `read_by_key` method

### tag offsets

key: `{namespace}_{shard}_tag_{tag}_offsets`

value: a list of offsets for all records in shard `shard` under namespace `namespace` whose `record.tags` contains `tag`

This key value pair is used for fast record retrieval in `read_by_tag` method

### group record offsets

key: `{group_name}_group`

value: A map of (`{namespace}_{shard}`, offset) pairs

This key value pair will be set in `commit_offset` method
