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

/// Represents a resolved NATS subscriber: a (connect_id, sid) pair matched to a concrete topic.
#[derive(Debug, Clone)]
pub struct NatsSubscriber {
    pub tenant: String,
    pub connect_id: u64,
    pub sid: String,
    /// Original subscription subject pattern (may contain wildcards).
    pub sub_subject: String,
    /// Concrete topic name matched against sub_subject.
    pub topic_name: String,
    /// Non-empty for queue-group subscriptions.
    pub queue_group: String,
    /// GroupConsumer group name: `nats_fanout_{connect_id}_{sid}_{topic_name}`.
    pub group_name: String,
    pub create_time: u64,
}

pub fn fanout_group_name(connect_id: u64, sid: &str, topic_name: &str) -> String {
    format!("nats_fanout_{}_{}_{}", connect_id, sid, topic_name)
}
