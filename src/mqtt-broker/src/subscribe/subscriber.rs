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

use protocol::mqtt::common::{Filter, MqttProtocol, QoS, RetainForwardRule, SubscribeProperties};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub protocol: MqttProtocol,
    pub client_id: String,
    pub sub_path: String,
    pub topic_name: String,
    pub group_name: Option<String>,
    pub topic_id: String,
    pub qos: QoS,
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub retain_forward_rule: RetainForwardRule,
    pub subscription_identifier: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscribeData {
    pub protocol: MqttProtocol,
    pub filter: Filter,
    pub subscribe_properties: Option<SubscribeProperties>,
}
