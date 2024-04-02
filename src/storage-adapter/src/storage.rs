pub trait StorageAdapter {
    fn kv_write(&self, key: String, value: String);
    fn kv_read(&self, key: String);
    fn kv_delete(&self, key: String);
    fn kv_exists(&self, key: String);
}
