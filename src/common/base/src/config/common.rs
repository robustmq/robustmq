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

use std::collections::HashMap;
use std::env;

use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Storage {
    pub storage_type: String,
    #[serde(default)]
    pub journal_addr: String,
    #[serde(default)]
    pub mysql_addr: String,
    #[serde(default)]
    pub rocksdb_data_path: String,
    pub rocksdb_max_open_files: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Auth {
    pub storage_type: String,
    #[serde(default)]
    pub journal_addr: String,
    #[serde(default)]
    pub mysql_addr: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Log {
    pub log_config: String,
    pub log_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Telemetry {
    pub enable: bool,
    pub exporter_type: String,
    pub exporter_endpoint: String,
}

/** `override_default_by_env` 根据环境变量覆盖内容

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
    // 逐行解析配置文件，生成环境变量键名与行号映射
    let mut sub_key = String::new(); // 当前子键
    let mut env_map = HashMap::new();
    for (line_num, line) in toml_content.lines().enumerate() {
        let trimmed = line.trim().replace(" ", "");
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue; // 跳过空行、表头或注释行
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
    // 遍历环境变量映射，查找并替换
    let mut lines: Vec<String> = toml_content.lines().map(|line| line.to_string()).collect();
    for (env_key, line_num) in &env_map {
        if let Ok(env_value) = env::var(env_key) {
            let key = lines[*line_num].split("=").collect::<Vec<&str>>()[0];
            lines[*line_num] = key.to_string() + "=" + &env_value;
        }
    }
    // 重新拼接修改后的 TOML 内容
    lines.join("\n")
}

#[cfg(test)]
mod tests {
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
}
