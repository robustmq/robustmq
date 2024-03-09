use std::sync::Arc;

use super::{keys::key_shard, rocksdb::RocksDBEngine};
use common::{config::placement_center::placement_center_conf, log::error_meta};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_id: String,
    pub shard_name: String,
    pub replica: u32,
    pub replicas: Vec<u64>,
    pub status: ShardStatus,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum ShardStatus {
    #[default]
    Idle,
    Write,
    PrepareSealUp,
    SealUp,
}

pub struct ShardStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShardStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let config = placement_center_conf();
        ShardStorage {
            rocksdb_engine_handler,
        }
    }

    // save shard info
    pub fn save_shard(&self, cluster_name: String, shard_info: ShardInfo) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let shard_key = key_shard(&cluster_name, shard_info.shard_name.clone());
        match self
            .rocksdb_engine_handler
            .write(cf, &shard_key, &shard_info)
        {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    // get shard info
    pub fn get_shard(&self, cluster_name: String, shard_name: String) -> Option<ShardInfo> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let shard_key: String = key_shard(&cluster_name, shard_name);
        match self
            .rocksdb_engine_handler
            .read::<ShardInfo>(cf, &shard_key)
        {
            Ok(ci) => {
                return ci;
            }
            Err(_) => {}
        }
        return None;
    }

    // delete shard info
    pub fn delete_shard(&self, cluster_name: String, shard_name: String) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let shard_key = key_shard(&cluster_name, shard_name);
        match self.rocksdb_engine_handler.delete(cf, &shard_key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }
}
