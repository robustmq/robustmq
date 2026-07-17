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

use dashmap::DashMap;
use metadata_struct::amqp::exchange::AmqpExchange;

// In-memory data the AMQP broker caches on every node. Populated at startup
// (see broker-server's load_amqp_cache) and kept current via the meta-service
// notify broadcast (send_notify_by_set_exchange / send_notify_by_delete_exchange)
// — nothing here ever calls meta-service directly on the read path.
#[derive(Default)]
pub struct AmqpCacheManager {
    // Exchanges, keyed by "{tenant}/{exchange_name}".
    exchanges: DashMap<String, AmqpExchange>,
}

impl AmqpCacheManager {
    pub fn new() -> Self {
        AmqpCacheManager {
            exchanges: DashMap::with_capacity(8),
        }
    }

    fn exchange_key(tenant: &str, exchange_name: &str) -> String {
        format!("{}/{}", tenant, exchange_name)
    }

    pub fn set_exchange(&self, exchange: AmqpExchange) {
        let key = Self::exchange_key(&exchange.tenant, &exchange.exchange_name);
        self.exchanges.insert(key, exchange);
    }

    pub fn remove_exchange(&self, tenant: &str, exchange_name: &str) {
        self.exchanges
            .remove(&Self::exchange_key(tenant, exchange_name));
    }

    pub fn get_exchange(&self, tenant: &str, exchange_name: &str) -> Option<AmqpExchange> {
        self.exchanges
            .get(&Self::exchange_key(tenant, exchange_name))
            .map(|e| e.clone())
    }

    pub fn list_exchanges_by_tenant(&self, tenant: &str) -> Vec<AmqpExchange> {
        let prefix = format!("{}/", tenant);
        self.exchanges
            .iter()
            .filter(|entry| entry.key().starts_with(&prefix))
            .map(|entry| entry.value().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::amqp::exchange::AmqpExchangeType;
    use std::collections::HashMap;

    fn exchange(tenant: &str, name: &str) -> AmqpExchange {
        AmqpExchange::new(
            tenant,
            name,
            AmqpExchangeType::Direct,
            true,
            false,
            false,
            HashMap::new(),
        )
    }

    #[test]
    fn set_get_remove_exchange() {
        let cache = AmqpCacheManager::new();
        cache.set_exchange(exchange("t1", "order.exchange"));
        assert!(cache.get_exchange("t1", "order.exchange").is_some());
        assert!(cache.get_exchange("t2", "order.exchange").is_none());

        cache.remove_exchange("t1", "order.exchange");
        assert!(cache.get_exchange("t1", "order.exchange").is_none());
    }

    #[test]
    fn list_exchanges_by_tenant_is_isolated() {
        let cache = AmqpCacheManager::new();
        cache.set_exchange(exchange("t1", "a"));
        cache.set_exchange(exchange("t1", "b"));
        cache.set_exchange(exchange("t2", "a"));

        assert_eq!(cache.list_exchanges_by_tenant("t1").len(), 2);
        assert_eq!(cache.list_exchanges_by_tenant("t2").len(), 1);
    }
}
