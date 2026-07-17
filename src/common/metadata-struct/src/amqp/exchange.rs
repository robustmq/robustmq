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

/// AMQP 0-9-1 exchange types (AMQPClass::Exchange's `exchange_type` is an
/// untyped ShortString on the wire; we validate it into this enum at the
/// protocol boundary rather than storing a free-form string).
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum AmqpExchangeType {
    #[default]
    Direct,
    Fanout,
    Topic,
    Headers,
}

impl AmqpExchangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmqpExchangeType::Direct => "direct",
            AmqpExchangeType::Fanout => "fanout",
            AmqpExchangeType::Topic => "topic",
            AmqpExchangeType::Headers => "headers",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "direct" => Some(AmqpExchangeType::Direct),
            "fanout" => Some(AmqpExchangeType::Fanout),
            "topic" => Some(AmqpExchangeType::Topic),
            "headers" => Some(AmqpExchangeType::Headers),
            _ => None,
        }
    }
}

/// An AMQP exchange. Pure routing metadata — unlike a Queue (which is backed by
/// a `metadata_struct::topic::Topic` shard), an exchange never stores messages
/// itself, so it has no storage/shard fields.
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct AmqpExchange {
    pub exchange_id: String,
    pub tenant: String,
    pub exchange_name: String,
    pub exchange_type: AmqpExchangeType,
    /// Metadata survives a broker restart when true (mirrors AMQP's own
    /// `durable` declare flag; independent of whether messages routed through
    /// it are persisted — that's controlled by delivery-mode on the message).
    pub durable: bool,
    /// Deleted once the last queue/exchange binding is removed.
    pub auto_delete: bool,
    /// An internal exchange can only be published to via exchange-to-exchange
    /// binding, never directly by a client's Basic.Publish.
    pub internal: bool,
    /// Declare-time arguments (AMQP FieldTable), e.g. "alternate-exchange".
    pub arguments: HashMap<String, String>,
    pub create_time: u64,
}

impl AmqpExchange {
    pub fn new(
        tenant: &str,
        exchange_name: &str,
        exchange_type: AmqpExchangeType,
        durable: bool,
        auto_delete: bool,
        internal: bool,
        arguments: HashMap<String, String>,
    ) -> Self {
        AmqpExchange {
            exchange_id: unique_id(),
            tenant: tenant.to_string(),
            exchange_name: exchange_name.to_string(),
            exchange_type,
            durable,
            auto_delete,
            internal,
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
        let exchange = AmqpExchange::new(
            "t1",
            "order.exchange",
            AmqpExchangeType::Topic,
            true,
            false,
            false,
            HashMap::new(),
        );
        let encoded = exchange.encode().unwrap();
        let decoded = AmqpExchange::decode(&encoded).unwrap();
        assert_eq!(exchange, decoded);
    }

    #[test]
    fn test_exchange_type_round_trip() {
        for t in [
            AmqpExchangeType::Direct,
            AmqpExchangeType::Fanout,
            AmqpExchangeType::Topic,
            AmqpExchangeType::Headers,
        ] {
            assert_eq!(AmqpExchangeType::from_str_opt(t.as_str()), Some(t));
        }
        assert_eq!(AmqpExchangeType::from_str_opt("bogus"), None);
    }
}
