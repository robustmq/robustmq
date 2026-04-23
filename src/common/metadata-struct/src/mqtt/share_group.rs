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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ShareGroupMember {
    pub tenant: String,
    pub group_name: String,
    pub broker_id: u64,
    pub connect_id: u64,
    pub create_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ShareGroupParams {
    MQTT(ShareGroupParamsMqtt),
    NATS(ShareGroupParamsNats),
    MQ9(ShareGroupParamsNats),
}

impl Default for ShareGroupParams {
    fn default() -> Self {
        ShareGroupParams::MQTT(ShareGroupParamsMqtt::default())
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ShareGroupParamsNats {}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ShareGroupParamsMqtt {
    path: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ShareGroup {
    pub uuid: String,
    pub tenant: String,
    pub group_name: String,
    pub leader_broker: u64,
    pub sub_params: ShareGroupParams,
    pub create_time: u64,
}

impl ShareGroup {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}
