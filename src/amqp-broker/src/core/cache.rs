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
use metadata_struct::amqp::binding::AmqpBinding;
use metadata_struct::amqp::exchange::AmqpExchange;
use metadata_struct::amqp::queue::AmqpQueue;

// In-memory data the AMQP broker caches on every node. Populated at startup
// (see broker-server's load_amqp_cache) and kept current via the meta-service
// notify broadcast (send_notify_by_set_exchange / send_notify_by_delete_exchange,
// and the queue/binding equivalents) — nothing here ever calls meta-service
// directly on the read path.
#[derive(Default)]
pub struct AmqpCacheManager {
    // Exchanges, keyed by "{tenant}/{exchange_name}".
    exchanges: DashMap<String, AmqpExchange>,
    // Queue declare metadata, keyed by "{tenant}/{queue_name}". The queue's
    // message shard (a Topic, TopicSource::AMQP) is tracked separately by
    // the shared broker_cache, not here.
    queues: DashMap<String, AmqpQueue>,
    // Bindings, keyed by "{tenant}/{binding.key()}".
    bindings: DashMap<String, AmqpBinding>,
}

impl AmqpCacheManager {
    pub fn new() -> Self {
        AmqpCacheManager {
            exchanges: DashMap::with_capacity(8),
            queues: DashMap::with_capacity(8),
            bindings: DashMap::with_capacity(8),
        }
    }

    fn tenant_name_key(tenant: &str, name: &str) -> String {
        format!("{}/{}", tenant, name)
    }

    pub fn set_exchange(&self, exchange: AmqpExchange) {
        let key = Self::tenant_name_key(&exchange.tenant, &exchange.exchange_name);
        self.exchanges.insert(key, exchange);
    }

    pub fn remove_exchange(&self, tenant: &str, exchange_name: &str) {
        self.exchanges
            .remove(&Self::tenant_name_key(tenant, exchange_name));
    }

    pub fn get_exchange(&self, tenant: &str, exchange_name: &str) -> Option<AmqpExchange> {
        self.exchanges
            .get(&Self::tenant_name_key(tenant, exchange_name))
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

    pub fn set_queue(&self, queue: AmqpQueue) {
        let key = Self::tenant_name_key(&queue.tenant, &queue.queue_name);
        self.queues.insert(key, queue);
    }

    pub fn remove_queue(&self, tenant: &str, queue_name: &str) {
        self.queues
            .remove(&Self::tenant_name_key(tenant, queue_name));
    }

    pub fn get_queue(&self, tenant: &str, queue_name: &str) -> Option<AmqpQueue> {
        self.queues
            .get(&Self::tenant_name_key(tenant, queue_name))
            .map(|q| q.clone())
    }

    pub fn list_queues_by_tenant(&self, tenant: &str) -> Vec<AmqpQueue> {
        let prefix = format!("{}/", tenant);
        self.queues
            .iter()
            .filter(|entry| entry.key().starts_with(&prefix))
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn set_binding(&self, binding: AmqpBinding) {
        let key = Self::tenant_name_key(&binding.tenant, &binding.key());
        self.bindings.insert(key, binding);
    }

    pub fn remove_binding(&self, tenant: &str, binding_key: &str) {
        self.bindings
            .remove(&Self::tenant_name_key(tenant, binding_key));
    }

    pub fn list_bindings_by_tenant(&self, tenant: &str) -> Vec<AmqpBinding> {
        let prefix = format!("{}/", tenant);
        self.bindings
            .iter()
            .filter(|entry| entry.key().starts_with(&prefix))
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn list_bindings_by_source(&self, tenant: &str, source: &str) -> Vec<AmqpBinding> {
        let prefix = format!("{}/{}/", tenant, source);
        self.bindings
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

    fn queue(tenant: &str, name: &str) -> AmqpQueue {
        AmqpQueue::new(tenant, name, true, false, false, HashMap::new())
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

    #[test]
    fn set_get_remove_queue() {
        let cache = AmqpCacheManager::new();
        cache.set_queue(queue("t1", "order.queue"));
        assert!(cache.get_queue("t1", "order.queue").is_some());
        assert!(cache.get_queue("t2", "order.queue").is_none());

        cache.remove_queue("t1", "order.queue");
        assert!(cache.get_queue("t1", "order.queue").is_none());
    }

    #[test]
    fn list_queues_by_tenant_is_isolated() {
        let cache = AmqpCacheManager::new();
        cache.set_queue(queue("t1", "a"));
        cache.set_queue(queue("t1", "b"));
        cache.set_queue(queue("t2", "a"));

        assert_eq!(cache.list_queues_by_tenant("t1").len(), 2);
        assert_eq!(cache.list_queues_by_tenant("t2").len(), 1);
    }

    fn binding(tenant: &str, source: &str, destination: &str, routing_key: &str) -> AmqpBinding {
        AmqpBinding::new(
            tenant,
            source,
            destination,
            metadata_struct::amqp::binding::AmqpBindingDestinationType::Queue,
            routing_key,
            HashMap::new(),
        )
    }

    #[test]
    fn set_remove_binding() {
        let cache = AmqpCacheManager::new();
        let b = binding("t1", "order.exchange", "order.queue", "order.created");
        cache.set_binding(b.clone());
        assert_eq!(cache.list_bindings_by_tenant("t1").len(), 1);

        cache.remove_binding("t1", &b.key());
        assert_eq!(cache.list_bindings_by_tenant("t1").len(), 0);
    }

    #[test]
    fn list_bindings_by_source_is_isolated() {
        let cache = AmqpCacheManager::new();
        cache.set_binding(binding(
            "t1",
            "order.exchange",
            "order.queue",
            "order.created",
        ));
        cache.set_binding(binding(
            "t1",
            "order.exchange",
            "audit.queue",
            "order.created",
        ));
        cache.set_binding(binding("t1", "other.exchange", "other.queue", "x"));

        assert_eq!(
            cache.list_bindings_by_source("t1", "order.exchange").len(),
            2
        );
        assert_eq!(
            cache.list_bindings_by_source("t1", "other.exchange").len(),
            1
        );
    }
}
