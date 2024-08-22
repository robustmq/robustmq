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

use crate::{
    config::{
        broker_mqtt::broker_mqtt_conf, journal_server::journal_server_conf,
        placement_center::placement_center_conf,
    },
    tools::{create_fold, file_exists, read_file},
};

pub fn init_placement_center_log() {
    let conf = placement_center_conf();
    init_log(&conf.log.log_config, &conf.log.log_path);
}

pub fn init_broker_mqtt_log() {
    let conf = broker_mqtt_conf();
    init_log(&conf.log.log_config, &conf.log.log_path);
}

pub fn init_journal_server_log() {
    let conf = journal_server_conf();
    init_log(&conf.log.log_config, &conf.log.log_path);
}

pub fn init_log(log_config_file: &String, log_path: &String) {
    if !file_exists(log_config_file) {
        panic!(
            "Logging configuration file {} does not exist",
            log_config_file
        );
    }

    let content = match read_file(&log_config_file) {
        Ok(data) => data,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };

    match create_fold(&log_path) {
        Ok(()) => {}
        Err(e) => {
            panic!("Failed to initialize log directory {}", log_path);
        }
    }

    let config_content = content.replace("{$path}", &log_path);
    let config = match serde_yaml::from_str(&config_content) {
        Ok(data) => data,
        Err(e) => {
            panic!(
                "Failed to parse the contents of the config file {} with error message :{}",
                log_config_file,
                e.to_string()
            );
        }
    };
    match log4rs::init_raw_config(config) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }
}

#[cfg(test)]
mod tests {}
