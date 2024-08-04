use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Storage {
    pub storage_type: String,
    #[serde(skip)]
    #[serde(default)]
    pub journal_addr: String,
    #[serde(skip)]
    #[serde(default)]
    pub mysql_addr: String,
}

impl Default for Storage {
    fn default() -> Self {
        return Storage {
            storage_type: "memory".to_string(),
            ..Default::default()
        };
    }
}
