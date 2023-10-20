// Copyright 2023 RobustMQ Team
//
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

use crate::log;
use serde::Deserialize;
use std::fs;
use std::path;
use toml;

pub const DEFAULT_SERVER_CONFIG: &str = "config/server.toml";

#[derive(Debug, Deserialize)]
pub struct RobustConfig {
    pub addr: String,
    pub broker: Broker,
    pub admin: Admin,
}

#[derive(Debug, Deserialize)]
pub struct Broker {
    pub port: Option<u16>,
    pub work_thread: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Admin {
    pub port: Option<u16>,
    pub work_thread: Option<u16>,
}


pub fn new(config_path: &String) -> RobustConfig {
    log::info(&format!("Configuration file path:{}.", config_path));

    if !path::Path::new(config_path).exists() {
        panic!("The configuration file does not exist.");
    }

    let content: String = fs::read_to_string(&config_path).expect(&format!(
        "Failed to read the configuration file. File path:{}.",
        config_path
    ));

    log::info(&format!(
        "server config content:\n============================\n{}\n============================\n",
        content
    ));

    let server_config: RobustConfig = toml::from_str(&content).unwrap();
    return server_config;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        
    }
}