use super::{keys::key_shard, rocksdb::RocksDBEngine};
use common::log::error_meta;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Clone)]
pub struct ShardStorage {
    rds: Arc<RocksDBEngine>,
}

impl ShardStorage {
    pub fn new(rds: Arc<RocksDBEngine>) -> Self {
        ShardStorage { rds }
    }

    // save shard info
    pub fn save_shard(&self, cluster_name: String, shard_info: ShardInfo) {
        let cf = self.rds.cf_cluster();
        let shard_key = key_shard(&cluster_name, shard_info.shard_name.clone());
        match self.rds.write(cf, &shard_key, &shard_info) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    // get shard info
    pub fn get_shard(&self, cluster_name: String, shard_name: String) -> Option<ShardInfo> {
        let cf = self.rds.cf_cluster();
        let shard_key: String = key_shard(&cluster_name, shard_name);
        match self.rds.read::<ShardInfo>(cf, &shard_key) {
            Ok(ci) => {
                return ci;
            }
            Err(_) => {}
        }
        return None;
    }

    // delete shard info
    pub fn delete_shard(&self, cluster_name: String, shard_name: String) {
        let cf = self.rds.cf_cluster();
        let shard_key = key_shard(&cluster_name, shard_name);
        match self.rds.delete(cf, &shard_key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }
}
