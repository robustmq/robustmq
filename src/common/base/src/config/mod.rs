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
use broker_mqtt::BrokerMQTTConfig;
use std::fs;
use std::path;
use toml;

pub mod broker_mqtt;
pub mod placement_center;
pub mod storage_engine;

pub const DEFAULT_BROKER_SERVER_CONFIG: &str = "config/broker-mqtt.toml";
pub const DEFAULT_PLACEMENT_CENTER_CONFIG: &str = "config/placement-center.toml";
pub const DEFAULT_STORAGE_ENGINE_CONFIG: &str = "config/storage-engine.toml";

/// Parsing reads the RobustMQ Server configuration
pub fn parse_server(config_path: &String) -> BrokerMQTTConfig {
    let content = read_file(config_path);
    let server_config: BrokerMQTTConfig = toml::from_str(&content).unwrap();
    return server_config;
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
