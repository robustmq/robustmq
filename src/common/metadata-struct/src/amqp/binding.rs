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

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum AmqpBindingDestinationType {
    #[default]
    Queue,
    Exchange,
}

impl AmqpBindingDestinationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmqpBindingDestinationType::Queue => "queue",
            AmqpBindingDestinationType::Exchange => "exchange",
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct AmqpBinding {
    pub binding_id: String,
    pub tenant: String,
    pub source: String,
    pub destination: String,
    pub destination_type: AmqpBindingDestinationType,
    pub routing_key: String,
    pub arguments: HashMap<String, String>,
    pub create_time: u64,
}

impl AmqpBinding {
    pub fn new(
        tenant: &str,
        source: &str,
        destination: &str,
        destination_type: AmqpBindingDestinationType,
        routing_key: &str,
        arguments: HashMap<String, String>,
    ) -> Self {
        AmqpBinding {
            binding_id: unique_id(),
            tenant: tenant.to_string(),
            source: source.to_string(),
            destination: destination.to_string(),
            destination_type,
            routing_key: routing_key.to_string(),
            arguments,
            create_time: now_second(),
        }
    }

    // (source, destination, routing_key) identifies a binding; arguments aren't
    // part of the identity.
    pub fn key(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.source,
            self.destination_type.as_str(),
            self.destination,
            self.routing_key
        )
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
        let binding = AmqpBinding::new(
            "t1",
            "order.exchange",
            "order.queue",
            AmqpBindingDestinationType::Queue,
            "order.created",
            HashMap::new(),
        );
        let encoded = binding.encode().unwrap();
        let decoded = AmqpBinding::decode(&encoded).unwrap();
        assert_eq!(binding, decoded);
    }
}
