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

use std::collections::{HashMap, HashSet};

use amq_protocol::types::{AMQPValue, FieldTable};
use metadata_struct::amqp::binding::{AmqpBinding, AmqpBindingDestinationType};
use metadata_struct::amqp::exchange::AmqpExchangeType;

use crate::core::cache::AmqpCacheManager;

/// Renders an AMQP field value as a plain string for routing comparisons
/// (headers-exchange matching, binding arguments). Strings render without
/// their Debug-format quoting; everything else falls back to Debug.
pub fn amqp_value_to_string(value: &AMQPValue) -> String {
    match value {
        AMQPValue::ShortString(s) => s.as_str().to_string(),
        AMQPValue::LongString(s) => s.to_string(),
        AMQPValue::Boolean(b) => b.to_string(),
        AMQPValue::ShortShortInt(n) => n.to_string(),
        AMQPValue::ShortShortUInt(n) => n.to_string(),
        AMQPValue::ShortInt(n) => n.to_string(),
        AMQPValue::ShortUInt(n) => n.to_string(),
        AMQPValue::LongInt(n) => n.to_string(),
        AMQPValue::LongUInt(n) => n.to_string(),
        AMQPValue::LongLongInt(n) => n.to_string(),
        AMQPValue::Float(n) => n.to_string(),
        AMQPValue::Double(n) => n.to_string(),
        AMQPValue::Timestamp(n) => n.to_string(),
        other => format!("{:?}", other),
    }
}

pub fn field_table_to_map(table: &FieldTable) -> HashMap<String, String> {
    table
        .inner()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), amqp_value_to_string(v)))
        .collect()
}

/// Topic-exchange wildcard match: `*` matches exactly one dot-separated word,
/// `#` matches zero or more words.
pub fn topic_match(pattern: &str, routing_key: &str) -> bool {
    let pattern_words: Vec<&str> = pattern.split('.').collect();
    let key_words: Vec<&str> = routing_key.split('.').collect();
    topic_match_words(&pattern_words, &key_words)
}

fn topic_match_words(pattern: &[&str], key: &[&str]) -> bool {
    match pattern.first() {
        None => key.is_empty(),
        Some(&"#") => {
            topic_match_words(&pattern[1..], key)
                || (!key.is_empty() && topic_match_words(pattern, &key[1..]))
        }
        Some(&"*") => !key.is_empty() && topic_match_words(&pattern[1..], &key[1..]),
        Some(word) => key.first() == Some(word) && topic_match_words(&pattern[1..], &key[1..]),
    }
}

/// Headers-exchange match: binding arguments other than `x-match` are the
/// match criteria; `x-match: any` requires one to match, anything else
/// (including absent) requires all of them to match.
fn headers_match(
    binding_arguments: &HashMap<String, String>,
    headers: &HashMap<String, String>,
) -> bool {
    let match_any = binding_arguments
        .get("x-match")
        .map(|v| v == "any")
        .unwrap_or(false);
    let criteria: Vec<(&String, &String)> = binding_arguments
        .iter()
        .filter(|(k, _)| k.as_str() != "x-match")
        .collect();
    if criteria.is_empty() {
        return !match_any;
    }
    if match_any {
        criteria.iter().any(|(k, v)| headers.get(*k) == Some(*v))
    } else {
        criteria.iter().all(|(k, v)| headers.get(*k) == Some(*v))
    }
}

fn binding_matches(
    exchange_type: &AmqpExchangeType,
    binding: &AmqpBinding,
    routing_key: &str,
    headers: &HashMap<String, String>,
) -> bool {
    match exchange_type {
        AmqpExchangeType::Direct => binding.routing_key == routing_key,
        AmqpExchangeType::Fanout => true,
        AmqpExchangeType::Topic => topic_match(&binding.routing_key, routing_key),
        AmqpExchangeType::Headers => headers_match(&binding.arguments, headers),
    }
}

/// Resolves a Basic.Publish (exchange, routing_key) into the concrete queue
/// names it should be written to, per the exchange's type and its bindings.
/// The empty-string exchange is the default exchange: an implicit direct
/// binding from every queue to itself by name, so it bypasses binding lookup
/// entirely. Exchange-to-exchange bindings are followed recursively, guarded
/// against cycles.
pub fn resolve_queues(
    cache: &AmqpCacheManager,
    tenant: &str,
    exchange_name: &str,
    routing_key: &str,
    headers: &HashMap<String, String>,
) -> Vec<String> {
    if exchange_name.is_empty() {
        return vec![routing_key.to_string()];
    }

    let mut visited = HashSet::new();
    let mut queues = Vec::new();
    resolve_inner(
        cache,
        tenant,
        exchange_name,
        routing_key,
        headers,
        &mut visited,
        &mut queues,
    );
    queues.sort();
    queues.dedup();
    queues
}

fn resolve_inner(
    cache: &AmqpCacheManager,
    tenant: &str,
    exchange_name: &str,
    routing_key: &str,
    headers: &HashMap<String, String>,
    visited: &mut HashSet<String>,
    queues: &mut Vec<String>,
) {
    if !visited.insert(exchange_name.to_string()) {
        return;
    }
    let Some(exchange) = cache.get_exchange(tenant, exchange_name) else {
        return;
    };
    for binding in cache.list_bindings_by_source(tenant, exchange_name) {
        if !binding_matches(&exchange.exchange_type, &binding, routing_key, headers) {
            continue;
        }
        match binding.destination_type {
            AmqpBindingDestinationType::Queue => queues.push(binding.destination.clone()),
            AmqpBindingDestinationType::Exchange => resolve_inner(
                cache,
                tenant,
                &binding.destination,
                routing_key,
                headers,
                visited,
                queues,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_match_star_matches_one_word() {
        assert!(topic_match("order.*.created", "order.eu.created"));
        assert!(!topic_match("order.*.created", "order.eu.west.created"));
    }

    #[test]
    fn topic_match_hash_matches_any_number_of_words() {
        assert!(topic_match("order.#", "order.eu.west.created"));
        assert!(topic_match("order.#", "order"));
        assert!(topic_match("#", "anything.at.all"));
    }

    #[test]
    fn headers_match_all_requires_every_criterion() {
        let mut binding_args = HashMap::new();
        binding_args.insert("x-match".to_string(), "all".to_string());
        binding_args.insert("region".to_string(), "eu".to_string());
        binding_args.insert("tier".to_string(), "gold".to_string());

        let mut headers = HashMap::new();
        headers.insert("region".to_string(), "eu".to_string());
        headers.insert("tier".to_string(), "gold".to_string());
        assert!(headers_match(&binding_args, &headers));

        headers.insert("tier".to_string(), "silver".to_string());
        assert!(!headers_match(&binding_args, &headers));
    }

    #[test]
    fn headers_match_any_requires_one_criterion() {
        let mut binding_args = HashMap::new();
        binding_args.insert("x-match".to_string(), "any".to_string());
        binding_args.insert("region".to_string(), "eu".to_string());
        binding_args.insert("tier".to_string(), "gold".to_string());

        let mut headers = HashMap::new();
        headers.insert("tier".to_string(), "gold".to_string());
        assert!(headers_match(&binding_args, &headers));

        headers.clear();
        assert!(!headers_match(&binding_args, &headers));
    }

    #[test]
    fn resolve_queues_default_exchange_is_direct_by_name() {
        let cache = AmqpCacheManager::new();
        let queues = resolve_queues(&cache, "t1", "", "order.queue", &HashMap::new());
        assert_eq!(queues, vec!["order.queue".to_string()]);
    }

    #[test]
    fn resolve_queues_fanout_ignores_routing_key() {
        use metadata_struct::amqp::exchange::AmqpExchange;

        let cache = AmqpCacheManager::new();
        cache.set_exchange(AmqpExchange::new(
            "t1",
            "fanout.ex",
            AmqpExchangeType::Fanout,
            true,
            false,
            false,
            HashMap::new(),
        ));
        cache.set_binding(AmqpBinding::new(
            "t1",
            "fanout.ex",
            "q1",
            AmqpBindingDestinationType::Queue,
            "",
            HashMap::new(),
        ));
        cache.set_binding(AmqpBinding::new(
            "t1",
            "fanout.ex",
            "q2",
            AmqpBindingDestinationType::Queue,
            "irrelevant",
            HashMap::new(),
        ));

        let mut queues = resolve_queues(&cache, "t1", "fanout.ex", "whatever", &HashMap::new());
        queues.sort();
        assert_eq!(queues, vec!["q1".to_string(), "q2".to_string()]);
    }

    #[test]
    fn resolve_queues_exchange_to_exchange_binding_is_followed() {
        use metadata_struct::amqp::exchange::AmqpExchange;

        let cache = AmqpCacheManager::new();
        cache.set_exchange(AmqpExchange::new(
            "t1",
            "src.ex",
            AmqpExchangeType::Direct,
            true,
            false,
            false,
            HashMap::new(),
        ));
        cache.set_exchange(AmqpExchange::new(
            "t1",
            "dst.ex",
            AmqpExchangeType::Fanout,
            true,
            false,
            false,
            HashMap::new(),
        ));
        cache.set_binding(AmqpBinding::new(
            "t1",
            "src.ex",
            "dst.ex",
            AmqpBindingDestinationType::Exchange,
            "order.created",
            HashMap::new(),
        ));
        cache.set_binding(AmqpBinding::new(
            "t1",
            "dst.ex",
            "q1",
            AmqpBindingDestinationType::Queue,
            "",
            HashMap::new(),
        ));

        let queues = resolve_queues(&cache, "t1", "src.ex", "order.created", &HashMap::new());
        assert_eq!(queues, vec!["q1".to_string()]);
    }

    #[test]
    fn resolve_queues_cyclic_exchange_binding_does_not_hang() {
        use metadata_struct::amqp::exchange::AmqpExchange;

        let cache = AmqpCacheManager::new();
        cache.set_exchange(AmqpExchange::new(
            "t1",
            "a.ex",
            AmqpExchangeType::Fanout,
            true,
            false,
            false,
            HashMap::new(),
        ));
        cache.set_exchange(AmqpExchange::new(
            "t1",
            "b.ex",
            AmqpExchangeType::Fanout,
            true,
            false,
            false,
            HashMap::new(),
        ));
        cache.set_binding(AmqpBinding::new(
            "t1",
            "a.ex",
            "b.ex",
            AmqpBindingDestinationType::Exchange,
            "",
            HashMap::new(),
        ));
        cache.set_binding(AmqpBinding::new(
            "t1",
            "b.ex",
            "a.ex",
            AmqpBindingDestinationType::Exchange,
            "",
            HashMap::new(),
        ));

        let queues = resolve_queues(&cache, "t1", "a.ex", "x", &HashMap::new());
        assert!(queues.is_empty());
    }
}
