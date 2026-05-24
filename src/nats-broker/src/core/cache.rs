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

use crate::core::connection::NatsConnection;
use broker_core::cache::NodeCacheManager;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::mq9::agent::MQ9Agent;
use metadata_struct::mq9::forward_rule::Mq9ForwardRule;
use metadata_struct::mq9::mail::MQ9Mail;
use metadata_struct::mq9::Priority;
use std::sync::Arc;

pub struct NatsCacheManager {
    pub node_cache: Arc<NodeCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub connection_info: DashMap<u64, NatsConnection>,
    /// Key: "{tenant}/{mail_address}"
    pub mail_info: DashMap<String, MQ9Mail>,
    /// Key: inbox subject, Value: sid
    pub inbox_data: DashMap<String, String>,
    /// Key: "{tenant}/{name}"
    pub agent_info: DashMap<String, MQ9Agent>,
    /// Forward rules indexed by tenant. Each tenant owns a list of rules
    /// that are evaluated in registration order on every send. Hot path,
    /// read-mostly: lookups happen inline on the send hot path, mutations
    /// only via admin API / metadata-change notifications.
    pub forward_rules: DashMap<String, Vec<Mq9ForwardRule>>,
}

impl NatsCacheManager {
    pub fn new(client_pool: Arc<ClientPool>, node_cache: Arc<NodeCacheManager>) -> Self {
        NatsCacheManager {
            node_cache,
            client_pool,
            connection_info: DashMap::with_capacity(1024),
            mail_info: DashMap::new(),
            inbox_data: DashMap::new(),
            agent_info: DashMap::new(),
            forward_rules: DashMap::new(),
        }
    }

    pub fn add_inbox(&self, inbox: String, sid: String) {
        self.inbox_data.insert(inbox, sid);
    }

    pub fn remove_inbox(&self, inbox: &str) {
        self.inbox_data.remove(inbox);
    }

    pub fn remove_inbox_by_sid(&self, sid: &str) {
        self.inbox_data.retain(|_, v| v != sid);
    }

    pub fn get_inbox_sid(&self, inbox: &str) -> Option<String> {
        self.inbox_data.get(inbox).map(|e| e.value().clone())
    }

    pub fn add_mail(&self, mail: MQ9Mail) {
        let key = format!("{}/{}", mail.tenant, mail.mail_address);
        self.mail_info.insert(key, mail);
    }

    pub fn get_mail(&self, tenant: &str, mail_address: &str) -> Option<MQ9Mail> {
        let key = format!("{}/{}", tenant, mail_address);
        self.mail_info.get(&key).map(|e| e.value().clone())
    }

    pub fn remove_mail(&self, tenant: &str, mail_address: &str) {
        let key = format!("{}/{}", tenant, mail_address);
        self.mail_info.remove(&key);
    }

    pub fn add_agent(&self, agent: MQ9Agent) {
        let key = format!("{}/{}", agent.tenant, agent.name);
        self.agent_info.insert(key, agent);
    }

    pub fn get_agent(&self, tenant: &str, name: &str) -> Option<MQ9Agent> {
        let key = format!("{}/{}", tenant, name);
        self.agent_info.get(&key).map(|e| e.value().clone())
    }

    pub fn remove_agent(&self, tenant: &str, name: &str) {
        let key = format!("{}/{}", tenant, name);
        self.agent_info.remove(&key);
    }

    // ---------- Mq9 forward rules ----------

    /// Insert or replace a forward rule. Rules are keyed by
    /// `(tenant, rule_name)`; same-name replaces in place.
    pub fn add_forward_rule(&self, rule: Mq9ForwardRule) {
        let mut entry = self.forward_rules.entry(rule.tenant.clone()).or_default();
        if let Some(pos) = entry.iter().position(|r| r.rule_name == rule.rule_name) {
            entry[pos] = rule;
        } else {
            entry.push(rule);
        }
    }

    pub fn remove_forward_rule(&self, tenant: &str, rule_name: &str) {
        if let Some(mut entry) = self.forward_rules.get_mut(tenant) {
            entry.retain(|r| r.rule_name != rule_name);
        }
    }

    /// Return all enabled rules under `tenant` whose matcher matches the
    /// supplied message dimensions. Returns `None` if no rule matches, so
    /// the caller can fast-path on the common case.
    ///
    /// `user_tags` must be the raw tag names from the `mq9-tags` header,
    /// NOT the `{tenant}/{addr}/{tag}` scoped form persisted on the record.
    pub fn match_forward_rules(
        &self,
        tenant: &str,
        mail_address: &str,
        user_tags: &[String],
        priority: &Priority,
    ) -> Option<Vec<Mq9ForwardRule>> {
        let entry = self.forward_rules.get(tenant)?;
        let matched: Vec<Mq9ForwardRule> = entry
            .iter()
            .filter(|r| r.enabled && r.matcher.matches(mail_address, user_tags, priority))
            .cloned()
            .collect();
        if matched.is_empty() {
            None
        } else {
            Some(matched)
        }
    }

    pub fn add_connection(&self, connection: NatsConnection) {
        self.connection_info
            .insert(connection.connect_id, connection);
    }

    pub fn remove_connection(&self, connect_id: u64) {
        self.connection_info.remove(&connect_id);
    }

    pub fn get_connection(&self, connect_id: u64) -> Option<NatsConnection> {
        self.connection_info
            .get(&connect_id)
            .map(|e| e.value().clone())
    }

    pub fn get_connection_count(&self) -> usize {
        self.connection_info.len()
    }

    pub fn login_success(&self, connect_id: u64, user_name: String) {
        if let Some(mut conn) = self.connection_info.get_mut(&connect_id) {
            conn.login_success(user_name);
        }
    }

    pub fn is_login(&self, connect_id: u64) -> bool {
        self.connection_info
            .get(&connect_id)
            .map(|e| e.is_login)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::default_broker_config;
    use metadata_struct::mq9::forward_rule::{
        ForkFailureStrategy, Mq9ForwardMatcher, Mq9ForwardRule, Mq9ForwardTarget,
    };

    fn cache() -> NatsCacheManager {
        NatsCacheManager::new(
            Arc::new(ClientPool::new(1)),
            Arc::new(NodeCacheManager::new(default_broker_config())),
        )
    }

    fn rule(tenant: &str, name: &str, matcher: Mq9ForwardMatcher, enabled: bool) -> Mq9ForwardRule {
        Mq9ForwardRule {
            tenant: tenant.to_string(),
            rule_name: name.to_string(),
            matcher,
            target: Mq9ForwardTarget {
                topic_name: format!("{}-out", name),
                keep_headers: true,
                on_failure: ForkFailureStrategy::DropAndLog,
            },
            etl_rule: None,
            enabled,
            create_time: 0,
            update_time: 0,
        }
    }

    #[test]
    fn add_then_match_returns_rule() {
        let c = cache();
        c.add_forward_rule(rule(
            "t1",
            "audit",
            Mq9ForwardMatcher {
                mail_address_prefixes: vec!["agent.".to_string()],
                ..Default::default()
            },
            true,
        ));

        let hit = c
            .match_forward_rules("t1", "agent.alice", &[], &Priority::Normal)
            .expect("rule should match");
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0].rule_name, "audit");

        let miss = c.match_forward_rules("t1", "user.alice", &[], &Priority::Normal);
        assert!(miss.is_none());
    }

    #[test]
    fn add_same_name_replaces_in_place() {
        let c = cache();
        c.add_forward_rule(rule("t1", "r", Mq9ForwardMatcher::default(), true));
        c.add_forward_rule(rule("t1", "r", Mq9ForwardMatcher::default(), false));

        // Disabled rules don't match — and we expect a single rule total,
        // not two.
        let entry = c.forward_rules.get("t1").unwrap();
        assert_eq!(entry.len(), 1);
        assert!(!entry[0].enabled);
        assert!(c
            .match_forward_rules("t1", "x", &[], &Priority::Normal)
            .is_none());
    }

    #[test]
    fn disabled_rule_is_skipped() {
        let c = cache();
        c.add_forward_rule(rule("t1", "r", Mq9ForwardMatcher::default(), false));
        assert!(c
            .match_forward_rules("t1", "anything", &[], &Priority::Normal)
            .is_none());
    }

    #[test]
    fn remove_rule_then_match_returns_none() {
        let c = cache();
        c.add_forward_rule(rule("t1", "r", Mq9ForwardMatcher::default(), true));
        assert!(c
            .match_forward_rules("t1", "x", &[], &Priority::Normal)
            .is_some());
        c.remove_forward_rule("t1", "r");
        assert!(c
            .match_forward_rules("t1", "x", &[], &Priority::Normal)
            .is_none());
    }

    #[test]
    fn match_is_tenant_scoped() {
        let c = cache();
        c.add_forward_rule(rule("t1", "r", Mq9ForwardMatcher::default(), true));
        assert!(c
            .match_forward_rules("t2", "anything", &[], &Priority::Normal)
            .is_none());
    }

    #[test]
    fn multiple_matching_rules_all_returned() {
        let c = cache();
        c.add_forward_rule(rule("t1", "a", Mq9ForwardMatcher::default(), true));
        c.add_forward_rule(rule(
            "t1",
            "b",
            Mq9ForwardMatcher {
                priorities: vec![Priority::Critical],
                ..Default::default()
            },
            true,
        ));
        let hits = c
            .match_forward_rules("t1", "x", &[], &Priority::Critical)
            .unwrap();
        assert_eq!(hits.len(), 2);
        let names: Vec<&str> = hits.iter().map(|r| r.rule_name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }
}
