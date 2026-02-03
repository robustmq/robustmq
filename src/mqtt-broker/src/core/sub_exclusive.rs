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

use crate::subscribe::manager::SubscribeManager;
use protocol::mqtt::common::Subscribe;
use std::sync::Arc;

const EXCLUSIVE_SUB_PREFIX: &str = "$exclusive";

pub fn is_exclusive_sub(sub_path: &str) -> bool {
    sub_path.starts_with(EXCLUSIVE_SUB_PREFIX)
}

pub fn decode_exclusive_sub_path_to_topic_name(sub_path: &str) -> &str {
    if is_exclusive_sub(sub_path) {
        sub_path.trim_start_matches(EXCLUSIVE_SUB_PREFIX)
    } else {
        sub_path
    }
}

pub fn allow_exclusive_subscribe(subscribe: &Subscribe) -> bool {
    for filter in subscribe.filters.clone() {
        if !is_exclusive_sub(&filter.path) {
            continue;
        }
    }
    true
}

pub fn already_exclusive_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
    subscribe: &Subscribe,
) -> bool {
    for filter in subscribe.filters.clone() {
        if !is_exclusive_sub(&filter.path) {
            continue;
        }
        let topic_name = decode_exclusive_sub_path_to_topic_name(&filter.path);
        if subscribe_manager.is_exclusive_subscribe_by_other(topic_name, client_id) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {

    use common_base::uuid::unique_id;
    use protocol::mqtt::common::{Filter, Subscribe};
    use std::sync::Arc;

    use crate::{
        core::sub_exclusive::{
            already_exclusive_subscribe, decode_exclusive_sub_path_to_topic_name, is_exclusive_sub,
        },
        subscribe::manager::SubscribeManager,
    };

    #[test]
    fn test_is_exclusive_sub() {
        assert!(is_exclusive_sub("$exclusive/topic"));
        assert!(is_exclusive_sub("$exclusive/another/topic"));
        assert!(!is_exclusive_sub("/exclusive/topic"));
        assert!(!is_exclusive_sub("/topic"));
    }

    #[test]
    fn test_decode_exclusive_sub_path_to_topic_name() {
        assert_eq!(
            decode_exclusive_sub_path_to_topic_name("$exclusive/topic"),
            "/topic"
        );
        assert_eq!(
            decode_exclusive_sub_path_to_topic_name("$exclusive/another/topic"),
            "/another/topic"
        );
        assert_eq!(decode_exclusive_sub_path_to_topic_name("topic"), "topic");
        assert_eq!(
            decode_exclusive_sub_path_to_topic_name("/exclusive/topic"),
            "/exclusive/topic"
        );
    }

    #[test]
    fn already_exclusive_subscribe_test() {
        let ex_path = "$exclusive/topic/1/2";
        let no_ex_path = "/no_topic/1/2";
        let topic_name = "/topic/1/2";
        let client_id = unique_id();

        let subscribe = Subscribe {
            packet_identifier: 0,
            filters: vec![
                Filter {
                    path: ex_path.to_string(),
                    ..Default::default()
                },
                Filter {
                    path: no_ex_path.to_string(),
                    ..Default::default()
                },
            ],
        };

        let subscribe_manager = Arc::new(SubscribeManager::new());
        assert!(!already_exclusive_subscribe(
            &subscribe_manager,
            &client_id,
            &subscribe
        ));

        subscribe_manager.add_topic_subscribe(topic_name, &client_id, ex_path);
        // Same client can resubscribe
        assert!(!already_exclusive_subscribe(
            &subscribe_manager,
            &client_id,
            &subscribe
        ));

        // Different client cannot subscribe
        let other_client_id = unique_id();
        assert!(already_exclusive_subscribe(
            &subscribe_manager,
            &other_client_id,
            &subscribe
        ))
    }
}
