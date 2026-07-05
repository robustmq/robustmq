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

use std::collections::{BTreeMap, BTreeSet, HashMap};

use uuid::Uuid;

use crate::core::consumer_group_meta::{ConsumerMemberMeta, TargetAssignment};

pub struct TopicMeta {
    pub topic_id: Uuid,
    pub partitions: u32,
}

// Range assignment: for each subscribed topic, its partitions are split into
// contiguous chunks across the subscribers (sorted by member id); the first
// `count % n` subscribers get one extra partition. Deterministic for a given
// membership so recomputing at the same epoch yields the same target.
pub(crate) fn compute_target(
    members: &HashMap<String, ConsumerMemberMeta>,
    resolve_topic: &dyn Fn(&str) -> Option<TopicMeta>,
) -> TargetAssignment {
    // topic_name -> sorted subscriber ids
    let mut subscribers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for member in members.values() {
        for topic in &member.subscribed {
            subscribers
                .entry(topic.clone())
                .or_default()
                .insert(member.member_id.clone());
        }
    }

    let mut target: TargetAssignment = HashMap::new();
    for (topic_name, subs) in subscribers {
        let Some(meta) = resolve_topic(&topic_name) else {
            continue;
        };
        let count = meta.partitions as i32;
        let n = subs.len() as i32;
        if count == 0 || n == 0 {
            continue;
        }
        let per = count / n;
        let extra = count % n;

        let mut next = 0;
        for (idx, member_id) in subs.iter().enumerate() {
            let take = per + if (idx as i32) < extra { 1 } else { 0 };
            if take == 0 {
                continue;
            }
            let partitions: Vec<i32> = (next..next + take).collect();
            next += take;
            target
                .entry(member_id.clone())
                .or_default()
                .insert(meta.topic_id, partitions);
        }
    }
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(id: &str, topics: Vec<&str>) -> (String, ConsumerMemberMeta) {
        (
            id.to_string(),
            ConsumerMemberMeta {
                member_id: id.to_string(),
                instance_id: None,
                rack_id: None,
                client_id: "c".to_string(),
                rebalance_timeout_ms: 60_000,
                subscribed: topics.into_iter().map(|t| t.to_string()).collect(),
                reported: HashMap::new(),
                member_epoch: 0,
                last_sent: None,
                last_heartbeat_ms: 0,
            },
        )
    }

    fn resolver(partitions: u32) -> impl Fn(&str) -> Option<TopicMeta> {
        move |name: &str| {
            Some(TopicMeta {
                topic_id: Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()),
                partitions,
            })
        }
    }

    #[test]
    fn range_splits_contiguously_with_remainder_to_first_members() {
        let members: HashMap<_, _> = [member("m1", vec!["t"]), member("m2", vec!["t"])].into();
        let resolve = resolver(5);
        let target = compute_target(&members, &resolve);

        let tid = Uuid::new_v5(&Uuid::NAMESPACE_OID, b"t");
        assert_eq!(target["m1"][&tid], vec![0, 1, 2]);
        assert_eq!(target["m2"][&tid], vec![3, 4]);
    }

    #[test]
    fn only_subscribers_get_a_topic() {
        let members: HashMap<_, _> = [member("m1", vec!["a"]), member("m2", vec!["b"])].into();
        let resolve = resolver(2);
        let target = compute_target(&members, &resolve);

        let ta = Uuid::new_v5(&Uuid::NAMESPACE_OID, b"a");
        let tb = Uuid::new_v5(&Uuid::NAMESPACE_OID, b"b");
        assert_eq!(target["m1"][&ta], vec![0, 1]);
        assert!(!target["m1"].contains_key(&tb));
        assert_eq!(target["m2"][&tb], vec![0, 1]);
    }

    #[test]
    fn unknown_topics_are_skipped() {
        let members: HashMap<_, _> = [member("m1", vec!["missing"])].into();
        let resolve = |_: &str| None;
        let target = compute_target(&members, &resolve);
        assert!(!target.contains_key("m1"));
    }
}
