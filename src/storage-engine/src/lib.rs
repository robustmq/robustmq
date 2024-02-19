use common::config::storage_engine::StorageEngineConfig;

mod storage;
mod services;
mod shard;
mod segment;
mod index;
mod record;
mod v1;
mod v2;

pub struct StorageEngine{

}

impl StorageEngine {
    
    pub fn new(config: StorageEngineConfig) -> Self{
        return StorageEngine{};
    }

    pub fn start(&self){

    }
}