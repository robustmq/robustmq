use serde::Deserialize;

#[derive(Debug, Deserialize,Clone)]
pub struct MetaConfig {
    pub addr: String,
    pub port: Option<u16>,
    pub node_id: i32,
    pub data_path: String,
    pub rocksdb: Rocksdb,
}

#[derive(Debug, Deserialize,Clone)]
pub struct Rocksdb {
    pub max_open_files: Option<i32>,
}

// Default basic configuration of meta cluster
const DEFAULT_META_ADDRESS: &str = "127.0.0.1";
const DEFAULT_META_PORT: Option<u16> = Some(1227);
const DEFAULT_DATA_PATH: &str = "/Users/wenqiangxu/data/robustmq";

// Default Settings for storage tier rocksdb
const DEFAULT_ROCKSDB_MAX_OPEN_FILES: Option<i32> = Some(10000);

impl Default for MetaConfig {
    fn default() -> Self {
        MetaConfig {
            addr: String::from(DEFAULT_META_ADDRESS),
            data_path:String::from(DEFAULT_DATA_PATH),
            port: DEFAULT_META_PORT,
            ..Default::default()
        }
    }
}

impl Default for Rocksdb {
    fn default() -> Self {
        Rocksdb {
            max_open_files: DEFAULT_ROCKSDB_MAX_OPEN_FILES,
        }
    }
}
