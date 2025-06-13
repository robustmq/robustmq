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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[derive(Serialize, Deserialize, PartialEq, Default, Clone, Debug)]
pub enum AvailableFlag {
    #[default]
    Disable,
    Enable,
}

// Prometheus
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Prometheus {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_prometheus_port")]
    pub port: u32,
    #[serde(default)]
    pub push_gateway_server: String,
    #[serde(default)]
    pub interval: u32,
    #[serde(default)]
    pub header: String,
}

// Log
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Log {
    pub log_config: String,
    pub log_path: String,
}

// Telemetry
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Telemetry {
    pub enable: bool,
    pub exporter_type: String,
    pub exporter_endpoint: String,
}

// Pprof
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Pprof {
    #[serde(default = "default_false")]
    pub enable: bool,
    #[serde(default = "default_pprof_port")]
    pub port: u16,
    #[serde(default = "default_pprof_frequency")]
    pub frequency: i32,
}

pub fn default_prometheus() -> Prometheus {
    Prometheus {
        enable: false,
        model: "pull".to_string(),
        port: default_prometheus_port(),
        push_gateway_server: "".to_string(),
        interval: 10,
        header: "".to_string(),
    }
}

pub fn default_pprof() -> Pprof {
    Pprof {
        enable: false,
        port: default_pprof_port(),
        frequency: default_pprof_frequency(),
    }
}

pub fn default_prometheus_port() -> u32 {
    9090
}

pub fn default_false() -> bool {
    false
}

pub fn default_pprof_port() -> u16 {
    6060
}

pub fn default_pprof_frequency() -> i32 {
    100
}

/** `override_default_by_env` Cover the content based on the environment variables

```
let toml_content = r#"
[server]
port = 8080
"#;
let env_prefix = "APP";
std::env::set_var("APP_SERVER_PORT", "8081");
let new_toml_content =override_default_by_env(toml_content, env_prefix);
assert_eq!(new_toml_content, "[server]\nport = 8081\n");
```
*/
pub fn override_default_by_env(toml_content: String, env_prefix: &str) -> String {
    let env_map = find_exist_env_for_config(&toml_content, env_prefix);

    let mut lines: Vec<String> = toml_content.lines().map(|line| line.to_string()).collect();
    for (env_key, line_num) in &env_map {
        if let Ok(env_value) = env::var(env_key) {
            let key = lines[*line_num].split("=").collect::<Vec<&str>>()[0];
            lines[*line_num] = key.to_string() + "=" + &env_value;
        }
    }

    lines.join("\n")
}

pub fn find_exist_env_for_config(toml_content: &str, env_prefix: &str) -> HashMap<String, usize> {
    let mut sub_key = String::new();
    let mut env_map = HashMap::new();

    for (line_num, line) in toml_content.lines().enumerate() {
        let trimmed = line.trim().replace(" ", "");
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            sub_key = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }
        if sub_key.is_empty() {
            let (key, _) = trimmed.split_once('=').unwrap();
            let env_key = format!("{}_{}", env_prefix, key.to_uppercase().replace('.', "_"));
            env_map.insert(env_key, line_num);
        } else {
            let (key, _) = trimmed.split_once('=').unwrap();
            let env_key = format!(
                "{}_{}_{}",
                env_prefix,
                sub_key.to_uppercase(),
                key.to_uppercase().replace('.', "_")
            );
            env_map.insert(env_key, line_num);
        }
    }

    env_map
}

#[cfg(test)]
mod tests {
    use crate::common::AvailableFlag;

    #[test]
    fn override_default_by_env() {
        let toml_content = r#"
        [server]
        port=8080
        "#;
        let env_prefix = "APP";
        std::env::set_var("APP_SERVER_PORT", "8081");
        let new_toml_content = super::override_default_by_env(toml_content.to_string(), env_prefix);
        assert_eq!(
            new_toml_content,
            r#"
        [server]
        port=8081
        "#
        );
    }

    #[test]
    fn client34_connect_test() {
        assert_eq!(AvailableFlag::Disable as u8, 0);
        assert_eq!(AvailableFlag::Enable as u8, 1);
    }
}
