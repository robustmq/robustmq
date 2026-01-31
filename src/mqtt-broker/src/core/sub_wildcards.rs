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

use crate::{core::error::MqttBrokerError, core::tool::ResultMqttBrokerError};

const SINGLE_LEVEL_WILDCARD: &str = "+";
const MULTI_LEVEL_WILDCARD: &str = "#";
const TOPIC_LEVEL_SEPARATOR: &str = "/";
const MAX_TOPIC_LENGTH: usize = 65535;

pub fn sub_path_validator(sub_path: &str) -> ResultMqttBrokerError {
    if sub_path.is_empty() {
        return Err(MqttBrokerError::InvalidSubPath(
            "topic filter cannot be empty".to_string(),
        ));
    }

    if sub_path.len() > MAX_TOPIC_LENGTH {
        return Err(MqttBrokerError::InvalidSubPath(
            "topic filter exceeds maximum length".to_string(),
        ));
    }

    if sub_path.contains('\0') {
        return Err(MqttBrokerError::InvalidSubPath(
            "topic filter cannot contain null character".to_string(),
        ));
    }

    let levels: Vec<&str> = sub_path.split(TOPIC_LEVEL_SEPARATOR).collect();
    let mut multi_level_wildcard_count = 0;

    for (index, level) in levels.iter().enumerate() {
        if level.contains(SINGLE_LEVEL_WILDCARD) && *level != SINGLE_LEVEL_WILDCARD {
            return Err(MqttBrokerError::InvalidSubPath(format!(
                "single-level wildcard '+' must occupy entire level: {}",
                sub_path
            )));
        }

        if level.contains(MULTI_LEVEL_WILDCARD) {
            multi_level_wildcard_count += 1;

            if *level != MULTI_LEVEL_WILDCARD {
                return Err(MqttBrokerError::InvalidSubPath(format!(
                    "multi-level wildcard '#' must occupy entire level: {}",
                    sub_path
                )));
            }

            if index != levels.len() - 1 {
                return Err(MqttBrokerError::InvalidSubPath(format!(
                    "multi-level wildcard '#' must be the last character: {}",
                    sub_path
                )));
            }
        }
    }

    if multi_level_wildcard_count > 1 {
        return Err(MqttBrokerError::InvalidSubPath(format!(
            "multi-level wildcard '#' can only appear once: {}",
            sub_path
        )));
    }

    Ok(())
}

pub fn is_wildcards(sub_path: &str) -> bool {
    sub_path.contains(SINGLE_LEVEL_WILDCARD) || sub_path.contains(MULTI_LEVEL_WILDCARD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_path_validator() {
        assert!(sub_path_validator("sensor/temperature").is_ok());
        assert!(sub_path_validator("sensor/+/temperature").is_ok());
        assert!(sub_path_validator("+/temperature").is_ok());
        assert!(sub_path_validator("sensor/+").is_ok());
        assert!(sub_path_validator("+").is_ok());
        assert!(sub_path_validator("sensor/#").is_ok());
        assert!(sub_path_validator("#").is_ok());
        assert!(sub_path_validator("sensor/+/+/temperature").is_ok());
        assert!(sub_path_validator("$SYS/broker/clients").is_ok());
        assert!(sub_path_validator("sport/tennis/player1/#").is_ok());
        assert!(sub_path_validator("/sensor/temperature").is_ok());
        assert!(sub_path_validator("sensor//temperature").is_ok());
        assert!(sub_path_validator("sensor/temperature/").is_ok());
        assert!(sub_path_validator("a/b/c/d/e/f/g").is_ok());
        assert!(sub_path_validator("/").is_ok());
        assert!(sub_path_validator("//").is_ok());
        assert!(sub_path_validator("+/+/+").is_ok());
        assert!(sub_path_validator("/+").is_ok());
        assert!(sub_path_validator("+/").is_ok());
        assert!(sub_path_validator("/#").is_ok());
        assert!(sub_path_validator("$SYS/#").is_ok());
        assert!(sub_path_validator("$SYS/broker/+/clients").is_ok());
        assert!(sub_path_validator("$share/group/topic").is_ok());

        assert!(sub_path_validator("").is_err());
        assert!(sub_path_validator("sensor+").is_err());
        assert!(sub_path_validator("+sensor").is_err());
        assert!(sub_path_validator("sen+sor").is_err());
        assert!(sub_path_validator("sensor/temp+").is_err());
        assert!(sub_path_validator("sensor/+temp/data").is_err());
        assert!(sub_path_validator("sensor/#/temperature").is_err());
        assert!(sub_path_validator("#/temperature").is_err());
        assert!(sub_path_validator("sensor/#/+").is_err());
        assert!(sub_path_validator("sensor#").is_err());
        assert!(sub_path_validator("#sensor").is_err());
        assert!(sub_path_validator("sensor/temp#").is_err());
        assert!(sub_path_validator("sensor/#data").is_err());
        assert!(sub_path_validator("#/#").is_err());
        assert!(sub_path_validator("sensor/#/other/#").is_err());
        assert!(sub_path_validator("sensor\0temperature").is_err());
        assert!(sub_path_validator("sensor/\0/temperature").is_err());
        assert!(sub_path_validator(&"a".repeat(MAX_TOPIC_LENGTH)).is_ok());
        assert!(sub_path_validator(&"a".repeat(MAX_TOPIC_LENGTH + 1)).is_err());

        assert!(is_wildcards("sensor/+/temperature"));
        assert!(is_wildcards("sensor/#"));
        assert!(is_wildcards("+"));
        assert!(is_wildcards("#"));
        assert!(is_wildcards("sensor/+/+"));
        assert!(!is_wildcards("sensor/temperature"));
        assert!(!is_wildcards("sensor"));
        assert!(!is_wildcards(""));
    }
}
