pub trait StorageFormat {
    fn create_segment(&self);
    fn append(&self);
    fn read(&self);
}

