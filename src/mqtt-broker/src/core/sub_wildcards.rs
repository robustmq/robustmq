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
use regex::Regex;

const SUBSCRIBE_WILDCARDS_1: &str = "+";
const SUBSCRIBE_WILDCARDS_2: &str = "#";
const SUBSCRIBE_SPLIT_DELIMITER: &str = "/";
const SUBSCRIBE_NAME_REGEX: &str = r"^[\$a-zA-Z0-9_#+/]+$";

pub fn sub_path_validator(sub_path: &str) -> ResultMqttBrokerError {
    let regex = Regex::new(SUBSCRIBE_NAME_REGEX)?;

    if !regex.is_match(sub_path) {
        return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
    }

    for path in sub_path.split(SUBSCRIBE_SPLIT_DELIMITER) {
        if path.contains(SUBSCRIBE_WILDCARDS_1) && path != SUBSCRIBE_WILDCARDS_1 {
            return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
        }
        if path.contains(SUBSCRIBE_WILDCARDS_2) && path != SUBSCRIBE_WILDCARDS_2 {
            return Err(MqttBrokerError::InvalidSubPath(sub_path.to_owned()));
        }
    }

    Ok(())
}

pub fn is_wildcards(sub_path: &str) -> bool {
    sub_path.contains(SUBSCRIBE_WILDCARDS_1) || sub_path.contains(SUBSCRIBE_WILDCARDS_2)
}

mod tests {}
