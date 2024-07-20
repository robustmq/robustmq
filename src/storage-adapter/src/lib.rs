pub mod journal;
pub mod memory;
pub mod mysql;
pub mod placement;
pub mod record;
pub mod storage;

pub const ADAPTER_NAME_PLACEMENT: &str = "placement";
pub const ADAPTER_NAME_JOURNAL: &str = "journal";
pub const ADAPTER_NAME_MEMORY: &str = "momory";
pub const ADAPTER_NAME_MYSQL: &str = "mysql";
