use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StorageEngineRecord {
    pub offset: i64,
    pub shard: Bytes,
    pub segment: u32,
    pub key: Option<Bytes>,
    pub data: Bytes,
    pub tags: Option<Vec<Bytes>>,
    pub create_t: u64,
}

impl StorageEngineRecord {

}
