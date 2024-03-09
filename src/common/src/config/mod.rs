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

use self::placement_center::PlacementCenterConfig;
use self::storage_engine::StorageEngineConfig;
use crate::tools::create_fold;
use broker_server::BrokerServerConfig;
use std::fs;
use std::path;
use toml;

pub mod placement_center;
pub mod broker_server;
pub mod storage_engine;

pub const DEFAULT_BROKER_SERVER_CONFIG: &str = "config/broker-server.toml";
pub const DEFAULT_PLACEMENT_CENTER_CONFIG: &str = "config/placement-center.toml";
pub const DEFAULT_STORAGE_ENGINE_CONFIG: &str = "config/storage-engine.toml";

/// Parsing reads the RobustMQ Server configuration
pub fn parse_server(config_path: &String) -> BrokerServerConfig {
    let content = read_file(config_path);
    let server_config: BrokerServerConfig = toml::from_str(&content).unwrap();
    return server_config;
}

/// Parsing reads the StorageEngine configuration
pub fn parse_storage_engine(config_path: &String) -> StorageEngineConfig {
    let content = read_file(config_path);
    let pc_config: StorageEngineConfig = toml::from_str(&content).unwrap();
    for fold in pc_config.data_path.clone() {
        create_fold(fold);
    }
    create_fold(pc_config.log_path.clone());
    return pc_config;
}


fn read_file(config_path: &String) -> String {
    if !path::Path::new(config_path).exists() {
        panic!("The configuration file does not exist.");
    }

    let content: String = fs::read_to_string(&config_path).expect(&format!(
        "Failed to read the configuration file. File path:{}.",
        config_path
    ));
    return content;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
