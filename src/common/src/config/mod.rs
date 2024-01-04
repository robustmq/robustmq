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

use server::RobustConfig;
use std::fs;
use std::path;
use toml;
use crate::tools::create_fold;
use self::meta::MetaConfig;

pub mod meta;
pub mod server;

pub const DEFAULT_SERVER_CONFIG: &str = "config/server.toml";
pub const DEFAULT_META_CONFIG: &str = "config/meta.toml";

/// Parsing reads the RobustMQ Server configuration
pub fn parse_server(config_path: &String) -> RobustConfig {
    let content = read_file(config_path);
    let server_config: RobustConfig = toml::from_str(&content).unwrap();
    return server_config;
}

/// Parsing reads the MetaService configuration
pub fn parse_meta(config_path: &String) -> MetaConfig {
    let content = read_file(config_path);
    let meta_config: MetaConfig = toml::from_str(&content).unwrap();
    create_fold(meta_config.data_path.clone());
    create_fold(meta_config.log_path.clone());
    return meta_config;
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
