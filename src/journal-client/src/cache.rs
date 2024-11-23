use common_base::error::common::CommonError;
use dashmap::DashMap;

pub struct MetadataCache {
    shards: DashMap<String, String>,
}

impl MetadataCache {
    pub fn new() -> Self {
        let shards = DashMap::with_capacity(8);
        MetadataCache { shards }
    }
}

pub fn load_metadata_cache() {}

fn load_shards_cahce() -> Result<(), CommonError> {
    
    Ok(())
}

fn load_node_cache() -> Result<(), CommonError> {
    Ok(())
}
