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

use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_base::uuid::unique_id;
use serde::{Deserialize, Serialize};

/// An AMQP queue's declare-time metadata. This is the AMQP-protocol-level
/// identity of the queue (does the declaration itself survive a restart?),
/// separate from the physical storage backing it: every queue — durable or
/// not — maps 1:1 to a `metadata_struct::topic::Topic` shard (TopicSource::AMQP)
/// that actually holds its messages while the queue is alive.
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct AmqpQueue {
    pub queue_id: String,
    pub tenant: String,
    pub queue_name: String,
    /// Metadata survives a broker restart when true (independent of message
    /// delivery-mode, which is a per-message concern handled by the storage
    /// layer, not this struct).
    pub durable: bool,
    /// Deleted once the connection that declared it closes.
    pub exclusive: bool,
    /// Deleted once its last consumer disconnects.
    pub auto_delete: bool,
    /// Declare-time arguments (AMQP FieldTable), e.g. "x-message-ttl".
    pub arguments: HashMap<String, String>,
    pub create_time: u64,
}

impl AmqpQueue {
    pub fn new(
        tenant: &str,
        queue_name: &str,
        durable: bool,
        exclusive: bool,
        auto_delete: bool,
        arguments: HashMap<String, String>,
    ) -> Self {
        AmqpQueue {
            queue_id: unique_id(),
            tenant: tenant.to_string(),
            queue_name: queue_name.to_string(),
            durable,
            exclusive,
            auto_delete,
            arguments,
            create_time: now_second(),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        Ok(serde_json::from_slice(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let queue = AmqpQueue::new("t1", "order.queue", true, false, false, HashMap::new());
        let encoded = queue.encode().unwrap();
        let decoded = AmqpQueue::decode(&encoded).unwrap();
        assert_eq!(queue, decoded);
    }
}
