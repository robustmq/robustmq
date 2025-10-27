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

use crate::{common::override_default_by_env, config::BrokerConfig};
use common_base::tools::{read_file, try_create_fold};
use std::sync::OnceLock;

static BROKER_MQTT_CONF: OnceLock<BrokerConfig> = OnceLock::new();

pub fn init_broker_conf_by_path(config_path: &str) -> &'static BrokerConfig {
    BROKER_MQTT_CONF.get_or_init(|| {
        let content = match read_file(config_path) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string())
            }
        };
        let new_content = override_default_by_env(content, "ROBUST_MQ_SERVER");
        let config: BrokerConfig = toml::from_str(&new_content).unwrap_or_else(|e| panic!("{}", e));
        match try_create_fold(&config.log.log_path) {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
        config
    })
}

pub fn init_broker_conf_by_config(config: BrokerConfig) -> &'static BrokerConfig {
    BROKER_MQTT_CONF.get_or_init(|| config)
}

pub fn broker_config() -> &'static BrokerConfig {
    match BROKER_MQTT_CONF.get() {
        Some(config) => config,
        None => {
            panic!("MQTT Broker configuration is not initialized, check the configuration file.");
        }
    }
}

pub fn default_broker_config() -> BrokerConfig {
    let config: BrokerConfig = toml::from_str(
        r#"
            cluster_name = 'test1'
            broker_id = 1
        "#,
    )
    .unwrap();
    config
}

pub fn default_rocksdb_family() -> String {
    "broker".to_string()
}
