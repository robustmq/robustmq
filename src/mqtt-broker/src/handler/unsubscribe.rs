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

use crate::subscribe::{
    sub_common::{decode_share_info, is_share_sub, path_regex_match},
    subscribe_manager::SubscribeManager,
};
use log::error;
use std::sync::Arc;

use super::cache::CacheManager;

pub fn stop_push_by_client_id(subscribe_manager: &Arc<SubscribeManager>, client_id: &str) {
    subscribe_manager.remove_exclusive_push_by_client_id(client_id);
    subscribe_manager.remove_share_subscribe_leader_by_client_id(client_id);
    subscribe_manager.remove_share_subscribe_follower_by_client_id(client_id);
}

pub fn unsubscribe_by_path(
    cache_manager: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
    filter_path: &[String],
) {
    for (topic_name, _) in cache_manager.topic_info.clone() {
        for path in filter_path {
            if !path_regex_match(topic_name.clone(), path.clone()) {
                continue;
            }

            if is_share_sub(path.clone()) {
                let (group_name, sub_name) = decode_share_info(path.clone());
                // share leader
                for (key, data) in subscribe_manager.share_leader_push.clone() {
                    let mut flag = false;
                    for (sub_key, share_sub) in data.sub_list {
                        if share_sub.client_id == *client_id
                            && (share_sub.group_name.is_some()
                                && share_sub.group_name.unwrap() == group_name)
                            && share_sub.sub_path == sub_name
                        {
                            let mut_data =
                                subscribe_manager.share_leader_push.get_mut(&key).unwrap();
                            mut_data.sub_list.remove(&sub_key);
                            subscribe_manager.remove_topic_subscribe_by_path(
                                &share_sub.topic_name,
                                &share_sub.sub_path,
                            );
                            flag = true;
                        }
                    }

                    if flag {
                        if let Some(sx) = subscribe_manager.share_leader_push_thread.get(&key) {
                            match sx.send(true) {
                                Ok(_) => {}
                                Err(e) => error!("{}", e),
                            }
                        }
                    }
                }

                // share follower
                for (key, data) in subscribe_manager.share_follower_resub.clone() {
                    if data.client_id == *client_id && data.filter.path == *path {
                        subscribe_manager.share_follower_resub.remove(&key);
                        if let Some(sx) = subscribe_manager.share_follower_resub_thread.get(&key) {
                            match sx.send(true) {
                                Ok(_) => {}
                                Err(e) => error!("{}", e),
                            }
                        }
                    }
                }
            } else {
                for (key, subscriber) in subscribe_manager.exclusive_push.clone() {
                    if subscriber.client_id == *client_id && subscriber.sub_path == *path {
                        if let Some(sx) = subscribe_manager.exclusive_push_thread.get(&key) {
                            match sx.send(true) {
                                Ok(_) => {}
                                Err(e) => error!("{}", e),
                            }
                            subscribe_manager.exclusive_push.remove(&key);
                        }
                        subscribe_manager.remove_topic_subscribe_by_path(
                            &subscriber.topic_name,
                            &subscriber.sub_path,
                        );
                    }
                }
            }
        }
    }
}
