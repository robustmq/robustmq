// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::{self, Path},
    time::{SystemTime, UNIX_EPOCH},
    u32,
};

use local_ip_address::local_ip;
use log::warn;
use uuid::Uuid;

use crate::error::common::CommonError;

pub fn create_fold(fold: &String) -> Result<(), CommonError> {
    if !Path::new(fold).exists() {
        fs::create_dir_all(fold)?
    }
    return Ok(());
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
            warn!(
                "If the local IP fails, stop the process.error message:{}",
                e.to_string()
            );
            return "127.0.0.1".to_string();
        }
    }
}

pub fn file_exists(path: &String) -> bool {
    return Path::new(path).exists();
}

pub fn read_file(path: &String) -> Result<String, CommonError> {
    if !path::Path::new(path).exists() {
        return Err(CommonError::CommmonError(format!(
            "File {} does not exist",
            path
        )));
    }

    return Ok(fs::read_to_string(&path)?);
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
