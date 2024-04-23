use std::{
    fs, mem::replace, path::Path, time::{SystemTime, UNIX_EPOCH}
};

use local_ip_address::local_ip;
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

pub fn now_second() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
}

pub fn unique_id() -> String {
    let uuid = Uuid::new_v4();
    return uuid.to_string().replace("-", "");
}

pub fn get_local_ip() -> String {
    match local_ip() {
        Ok(data) => {
            return data.to_string();
        }
        Err(e) => {
            panic!(
                "If the local IP fails, stop the process.error message:{}",
                e.to_string()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::{get_local_ip, unique_id};

    #[test]
    fn get_local_ip_test() {
        println!("{}", get_local_ip());
    }

    #[test]
    fn unique_id_test() {
        println!("{}", unique_id());
    }
}
