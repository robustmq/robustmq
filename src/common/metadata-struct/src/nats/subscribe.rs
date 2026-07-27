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

use common_base::{error::common::CommonError, utils::serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NatsSubscribe {
    pub broker_id: u64,
    pub tenant: String,
    pub connect_id: u64,
    pub sid: String,
    pub subject: String,
    pub queue_group: Option<String>,
    pub create_time: u64,
    /// Per-shard offsets snapshotted, at SUB processing time, for every topic
    /// already matching this subscription's subject/pattern (topic_name ->
    /// shard_name -> end_offset). Fanout registration (which can happen much
    /// later than this SUB was accepted, since it's driven by an async
    /// raft-broadcast round trip) uses this to pin "latest" to what it
    /// actually meant at subscribe time, instead of whatever the topic's tail
    /// happens to be whenever the match finally occurs. A topic absent from
    /// this map didn't exist yet at subscribe time, so it correctly starts
    /// from the very beginning once created — see `register_subscriber`.
    #[serde(default)]
    pub known_topic_offsets: HashMap<String, HashMap<String, u64>>,
}

impl NatsSubscribe {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}
