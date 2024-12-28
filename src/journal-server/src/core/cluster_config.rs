#[derive(Clone, Default)]
pub struct JournalEngineClusterConfig {
    pub enable_auto_create_shard: bool,
    pub default_shard_replica_num: u32,
    pub last_update_local_cache_time: u64,
}

impl JournalEngineClusterConfig {
    pub fn new() -> Self {
        JournalEngineClusterConfig {
            enable_auto_create_shard: true,
            default_shard_replica_num: 2,
            last_update_local_cache_time: 0,
        }
    }
}
