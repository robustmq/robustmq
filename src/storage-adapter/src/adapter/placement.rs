use crate::storage::StorageAdapter;

pub struct PlacementStorageAdapter {}

impl PlacementStorageAdapter {
    pub fn new() -> Self {
        return PlacementStorageAdapter {};
    }
}

impl StorageAdapter for PlacementStorageAdapter {
    fn kv_write(&self, key: String, value: String) {
        
    }
    fn kv_read(&self, key: String) {}
    fn kv_delete(&self, key: String) {}
    fn kv_exists(&self, key: String) {}
}
