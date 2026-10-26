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

// Log
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Log {
    #[serde(default = "default_log_config")]
    pub log_config: String,
    #[serde(default = "default_log_path")]
    pub log_path: String,
}

impl Default for Log {
    fn default() -> Self {
        default_log()
    }
}

// Telemetry
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Telemetry {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub exporter_type: String,
    #[serde(default)]
    pub exporter_endpoint: String,
}

pub fn default_log_config() -> String {
    "./config/logger.toml".to_string()
}
pub fn default_log_path() -> String {
    "./logs".to_string()
}

pub fn default_false() -> bool {
    false
}

pub fn default_log() -> Log {
    Log {
        log_path: default_log_path(),
        log_config: default_log_config(),
    }
}

/** `override_default_by_env` Cover the content based on the environment variables

```
use common_config::common::override_default_by_env;

let toml_content = r#"[server]
port=8080
"#;
let env_prefix = "APP";
std::env::set_var("APP_SERVER_PORT", "8081");
let new_toml_content = override_default_by_env(toml_content.to_string(), env_prefix);
assert_eq!(new_toml_content, "[server]\nport=8081");
```
*/
pub fn override_default_by_env(toml_content: String, env_prefix: &str) -> String {
    let env_map = find_exist_env_for_config(&toml_content, env_prefix);

    let mut lines: Vec<String> = toml_content.lines().map(|line| line.to_string()).collect();
    for (env_key, line_num) in &env_map {
        if let Ok(env_value) = env::var(env_key) {
            let line = &lines[*line_num];
            let (key, _) = line.split_once('=').unwrap_or((line.as_str(), ""));
            lines[*line_num] = format!("{}={}", key, env_value);
        }
    }

    lines.join("\n")
}

pub fn find_exist_env_for_config(toml_content: &str, env_prefix: &str) -> HashMap<String, usize> {
    let mut sub_key = String::new();
    let mut env_map = HashMap::new();

    for (line_num, line) in toml_content.lines().enumerate() {
        let trimmed = line.trim().replace(' ', "");
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            sub_key = trimmed
                .strip_suffix(']')
                .map(|s| s[1..].to_string())
                .unwrap_or_else(|| trimmed[1..].to_string());
            continue;
        }
        let Some((key, _)) = trimmed.split_once('=') else {
            continue;
        };
        let key_upper = key.trim().to_uppercase().replace('.', "_");
        let env_key = if sub_key.is_empty() {
            format!("{}_{}", env_prefix, key_upper)
        } else {
            let section_upper = sub_key.to_uppercase().replace('.', "_");
            format!("{}_{}_{}", env_prefix, section_upper, key_upper)
        };
        env_map.insert(env_key, line_num);
    }

    env_map
}

#[cfg(test)]
mod tests {
    use crate::common::{
        default_false, default_log, default_log_config, default_log_path,
        find_exist_env_for_config, AvailableFlag,
    };

    #[test]
    fn default_log_helpers_match_the_default_value() {
        let log = default_log();
        assert_eq!(log.log_config, default_log_config());
        assert_eq!(log.log_path, default_log_path());
        assert!(!default_false());
    }

    #[test]
    fn find_exist_env_for_config_handles_nested_sections_and_comments() {
        let config = "# ignored\nroot = 1\n[server]\nport = 1883\n[server.tls]\nenabled = true\n";
        let env_map = find_exist_env_for_config(config, "APP");

        assert_eq!(env_map.get("APP_ROOT"), Some(&1));
        assert_eq!(env_map.get("APP_SERVER_PORT"), Some(&3));
        assert_eq!(env_map.get("APP_SERVER_TLS_ENABLED"), Some(&5));
        assert_eq!(env_map.len(), 3);
    }

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
