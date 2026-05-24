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

use crate::connector::rule::ETLRule;
use crate::mq9::Priority;
use common_base::{error::common::CommonError, utils::serialize};
use serde::{Deserialize, Serialize};

/// Strategy applied when a fork write fails.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForkFailureStrategy {
    /// Log the error, increment a metric, and continue. Mailbox write is unaffected.
    /// This is the default — keeps the agent send path latency-stable.
    #[default]
    DropAndLog,
    /// Propagate the fork-write error to the caller. The mailbox write has already
    /// succeeded, so the caller will see a failure even though the original message
    /// is durable in the mailbox; on retry, the mailbox will contain a duplicate.
    /// Use only when the fork target is more important than send-path liveness.
    FailSend,
}

/// Predicate that decides whether an incoming message should be forked.
///
/// All non-empty dimensions must match (logical AND). An empty dimension is a
/// wildcard. `any_tags` is a logical OR within the dimension: at least one of
/// the configured tag names must appear on the message.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mq9ForwardMatcher {
    /// Match if the mail address starts with any of these prefixes.
    #[serde(default)]
    pub mail_address_prefixes: Vec<String>,
    /// Match if the message carries at least one of these user tags (not
    /// the system-internal `{tenant}/{addr}/{tag}` scoped form — the raw tag
    /// names supplied via the `mq9-tags` header).
    #[serde(default)]
    pub any_tags: Vec<String>,
    /// Match if the message priority is in this set.
    #[serde(default)]
    pub priorities: Vec<Priority>,
    /// Reserved for future use: match if the sender address starts with any
    /// of these prefixes. Currently not evaluated (send-path does not carry
    /// a sender identity).
    #[serde(default)]
    pub sender_prefixes: Vec<String>,
}

impl Mq9ForwardMatcher {
    /// Returns true if the matcher has no predicates set. An empty matcher
    /// matches everything.
    pub fn is_wildcard(&self) -> bool {
        self.mail_address_prefixes.is_empty()
            && self.any_tags.is_empty()
            && self.priorities.is_empty()
            && self.sender_prefixes.is_empty()
    }

    /// Evaluate the matcher against an inbound message.
    ///
    /// `user_tags` should be the raw tag names from `mq9-tags`, NOT the
    /// `{tenant}/{addr}/{tag}` scoped form persisted on the record.
    pub fn matches(&self, mail_address: &str, user_tags: &[String], priority: &Priority) -> bool {
        if !self.mail_address_prefixes.is_empty()
            && !self
                .mail_address_prefixes
                .iter()
                .any(|p| mail_address.starts_with(p))
        {
            return false;
        }

        if !self.any_tags.is_empty() && !self.any_tags.iter().any(|t| user_tags.contains(t)) {
            return false;
        }

        if !self.priorities.is_empty() && !self.priorities.contains(priority) {
            return false;
        }

        true
    }
}

/// Destination of a forked message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mq9ForwardTarget {
    /// Target topic to write the forked message into. Any existing connector
    /// or consumer group subscribed to this topic will receive the fork.
    pub topic_name: String,
    /// When true, the original NATS headers and the source mq9 protocol
    /// metadata (priority, reply_to, raw header block) are preserved on the
    /// forked record. When false, only the payload is forwarded.
    #[serde(default = "default_true")]
    pub keep_headers: bool,
    /// What to do if the fork write itself fails.
    #[serde(default)]
    pub on_failure: ForkFailureStrategy,
}

fn default_true() -> bool {
    true
}

/// A user-configured rule that duplicates messages from a mailbox into a
/// secondary topic for downstream processing (auditing, RL, analytics,
/// LangChain pipelines, ...).
///
/// Each `(tenant, rule_name)` is globally unique.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Mq9ForwardRule {
    pub tenant: String,
    pub rule_name: String,
    pub matcher: Mq9ForwardMatcher,
    pub target: Mq9ForwardTarget,
    /// Optional payload transform applied before the fork write. When None,
    /// the original payload bytes are forwarded verbatim.
    #[serde(default)]
    pub etl_rule: Option<ETLRule>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub create_time: u64,
    pub update_time: u64,
}

impl Mq9ForwardRule {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(matcher: Mq9ForwardMatcher) -> Mq9ForwardRule {
        Mq9ForwardRule {
            tenant: "default".to_string(),
            rule_name: "test".to_string(),
            matcher,
            target: Mq9ForwardTarget {
                topic_name: "audit.fork".to_string(),
                keep_headers: true,
                on_failure: ForkFailureStrategy::default(),
            },
            etl_rule: None,
            enabled: true,
            create_time: 0,
            update_time: 0,
        }
    }

    #[test]
    fn wildcard_matches_anything() {
        let m = Mq9ForwardMatcher::default();
        assert!(m.is_wildcard());
        assert!(m.matches("alice", &[], &Priority::Normal));
        assert!(m.matches("any.address", &["billing".to_string()], &Priority::Critical));
    }

    #[test]
    fn mail_address_prefix_match() {
        let m = Mq9ForwardMatcher {
            mail_address_prefixes: vec!["agent.".to_string()],
            ..Default::default()
        };
        assert!(m.matches("agent.alice", &[], &Priority::Normal));
        assert!(!m.matches("user.alice", &[], &Priority::Normal));
    }

    #[test]
    fn any_tags_is_or_within_dimension() {
        let m = Mq9ForwardMatcher {
            any_tags: vec!["billing".to_string(), "audit".to_string()],
            ..Default::default()
        };
        assert!(m.matches("a", &["audit".to_string()], &Priority::Normal));
        assert!(m.matches(
            "a",
            &["billing".to_string(), "other".to_string()],
            &Priority::Normal
        ));
        assert!(!m.matches("a", &["other".to_string()], &Priority::Normal));
        assert!(!m.matches("a", &[], &Priority::Normal));
    }

    #[test]
    fn priority_filter() {
        let m = Mq9ForwardMatcher {
            priorities: vec![Priority::Critical, Priority::Urgent],
            ..Default::default()
        };
        assert!(m.matches("a", &[], &Priority::Critical));
        assert!(m.matches("a", &[], &Priority::Urgent));
        assert!(!m.matches("a", &[], &Priority::Normal));
    }

    #[test]
    fn all_dimensions_are_anded() {
        let m = Mq9ForwardMatcher {
            mail_address_prefixes: vec!["agent.".to_string()],
            any_tags: vec!["billing".to_string()],
            priorities: vec![Priority::Critical],
            ..Default::default()
        };
        // hits every dimension
        assert!(m.matches("agent.alice", &["billing".to_string()], &Priority::Critical));
        // misses on priority
        assert!(!m.matches("agent.alice", &["billing".to_string()], &Priority::Normal));
        // misses on tag
        assert!(!m.matches("agent.alice", &[], &Priority::Critical));
        // misses on address prefix
        assert!(!m.matches("user.alice", &["billing".to_string()], &Priority::Critical));
    }

    #[test]
    fn encode_decode_roundtrip() {
        let r = rule(Mq9ForwardMatcher {
            mail_address_prefixes: vec!["agent.".to_string()],
            any_tags: vec!["audit".to_string()],
            priorities: vec![Priority::Critical],
            ..Default::default()
        });
        let bytes = r.encode().unwrap();
        let r2 = Mq9ForwardRule::decode(&bytes).unwrap();
        assert_eq!(r, r2);
    }

    #[test]
    fn default_failure_strategy_is_drop_and_log() {
        assert_eq!(
            ForkFailureStrategy::default(),
            ForkFailureStrategy::DropAndLog
        );
    }
}
