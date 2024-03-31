use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use snowflake::ProcessUniqueId;
use uuid::Uuid;

pub fn create_fold(fold: String) {
    if !Path::new(&fold).exists() {
        fs::create_dir_all(fold).unwrap();
    }
}

pub fn now_mills() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
}

pub fn unique_id() -> String {
    let uuid = Uuid::new_v4();
    return uuid.to_string();
}

pub fn unique_id_string() -> String {
    let res = ProcessUniqueId::new();

    return res.to_string();
}
