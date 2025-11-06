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

use common_base::{error::common::CommonError, utils::serialize};
use protocol::mqtt::common::{Filter, MqttProtocol, SubscribeProperties};
use serde::{Deserialize, Serialize};

pub const SHARE_SUB_PREFIX: &str = "$share";
pub const QUEUE_SUB_PREFIX: &str = "$queue";

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct MqttSubscribe {
    pub client_id: String,
    pub path: String,
    pub cluster_name: String,
    pub broker_id: u64,
    pub protocol: MqttProtocol,
    pub filter: Filter,
    pub pkid: u16,
    pub subscribe_properties: Option<SubscribeProperties>,
    pub create_time: u64,
}

impl MqttSubscribe {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

pub fn is_mqtt_share_subscribe(sub_name: &str) -> bool {
    is_mqtt_share_sub(sub_name) || is_mqtt_queue_sub(sub_name)
}

pub fn is_mqtt_share_sub(sub_name: &str) -> bool {
    sub_name.starts_with(SHARE_SUB_PREFIX)
}

pub fn is_mqtt_queue_sub(sub_name: &str) -> bool {
    sub_name.starts_with(QUEUE_SUB_PREFIX)
}

#[cfg(test)]
mod tests {
    use crate::mqtt::subscribe_data::{is_mqtt_share_sub, is_mqtt_share_subscribe};

    #[test]
    fn is_mqtt_share_subscribe_test() {
        assert!(is_mqtt_share_sub("$share/g1/test/hello"));
        assert!(is_mqtt_share_subscribe("$share/g1/test/hello"));
        assert!(!is_mqtt_share_subscribe("/test/hello"));
    }
}
