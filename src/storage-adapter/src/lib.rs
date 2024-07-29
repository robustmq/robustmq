pub mod journal;
pub mod memory;
pub mod mysql;
pub mod placement;
pub mod record;
pub mod storage;

#[derive(Debug)]
pub enum StorageType {
    Journal,
    Memory,
    Mysql,
}

pub fn storage_is_journal(storage_type: &String) -> bool {
    let st = format!("{:?}", StorageType::Journal).to_lowercase();
    return st == storage_type.clone();
}

pub fn storage_is_memory(storage_type: &String) -> bool {
    let st = format!("{:?}", StorageType::Memory).to_lowercase();
    return st == storage_type.clone();
}

pub fn storage_is_mysql(storage_type: &String) -> bool {
    let st = format!("{:?}", StorageType::Mysql).to_lowercase();
    return st == storage_type.clone();
}

#[cfg(test)]
mod tests {
    use crate::{storage_is_journal, storage_is_memory, storage_is_mysql};

    #[tokio::test]
    async fn storage_type_test() {
        assert!(storage_is_journal(&"journal".to_string()));
        assert!(storage_is_memory(&"memory".to_string()));
        assert!(storage_is_mysql(&"mysql".to_string()));
    }
}
