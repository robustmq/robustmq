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
use common_base::utils::topic_util::{decode_exclusive_sub_path_to_topic_name, is_exclusive_sub};
use protocol::mqtt::common::Subscribe;
use std::sync::Arc;

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
    subscribe: &Subscribe,
) -> bool {
    for filter in subscribe.filters.clone() {
        if !is_exclusive_sub(&filter.path) {
            continue;
        }
        let topic_name = decode_exclusive_sub_path_to_topic_name(&filter.path);
        if subscribe_manager.is_exclusive_subscribe(topic_name) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {

    use common_base::tools::unique_id;
    use protocol::mqtt::common::{Filter, Subscribe};
    use std::sync::Arc;

    use crate::{
        handler::sub_exclusive::already_exclusive_subscribe, subscribe::manager::SubscribeManager,
    };

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
        assert!(!already_exclusive_subscribe(&subscribe_manager, &subscribe));

        subscribe_manager.add_topic_subscribe(topic_name, &client_id, ex_path);
        assert!(already_exclusive_subscribe(&subscribe_manager, &subscribe))
    }
}
