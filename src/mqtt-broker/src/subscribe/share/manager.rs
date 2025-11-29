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

use crate::subscribe::manager::{ShareLeaderSubscribeData, SubPushThreadData};

pub struct ShareSubManager {
    // (group_name_topic_name, ShareLeaderSubscribeData)
    share_push: DashMap<String, ShareLeaderSubscribeData>,

    // (group_name_topic_name, SubPushThreadData)
    share_push_thread: DashMap<String, SubPushThreadData>,
}
impl ShareSubManager {
    pub fn new() -> Self {
        ShareSubManager {
            share_push: DashMap::with_capacity(8),
            share_push_thread: DashMap::with_capacity(8),
        }
    }
    
    // share push by share subscribe
    pub fn share_push_list(&self) -> DashMap<String, ShareLeaderSubscribeData> {
        self.share_push.clone()
    }

    pub fn share_push_len(&self) -> u64 {
        self.share_push.len() as u64
    }

    pub fn contain_share_push(&self, share_leader_key: &str) -> bool {
        self.share_push.contains_key(share_leader_key)
    }

    pub fn remove_share_push(&self, share_leader_key: &str) {
        self.share_push.remove(share_leader_key);
    }

    pub fn get_share_push(&self, share_leader_key: &str) -> Option<ShareLeaderSubscribeData> {
        if let Some(data) = self.share_push.get(share_leader_key) {
            return Some(data.clone());
        }
        None
    }

    pub fn add_share_subscribe(&self, sub_path: &str, sub: Subscriber) {
        let group_name = sub.group_name.clone().unwrap();
        let share_leader_key = self.share_leader_key(&group_name, sub_path, &sub.topic_name);

        // add topic-sub, sub-topic
        self.add_subscribe_topics(&sub.client_id, &sub.sub_path, &sub.topic_name);
        self.add_topic_subscribe(&sub.topic_name, &sub.client_id, &sub.sub_path);

        // add leader push data
        if let Some(share_sub) = self.share_push.get(&share_leader_key) {
            share_sub
                .sub_list
                .insert(sub.client_id.clone(), sub.clone());
        } else {
            let sub_list = DashMap::with_capacity(2);
            sub_list.insert(sub.client_id.clone(), sub.clone());
            self.share_push.insert(
                share_leader_key.clone(),
                ShareLeaderSubscribeData {
                    group_name: group_name.to_owned(),
                    topic_name: sub.topic_name.to_owned(),
                    sub_path: sub_path.to_owned(),
                    path: sub.sub_path,
                    sub_list,
                },
            );
        }
    }

    fn remove_share_subscribe_by_client_id(&self, client_id: &str) {
        for share_sub in self.share_push.iter() {
            if let Some((_, subscriber)) = share_sub.sub_list.remove(client_id) {
                self.remove_topic_subscribe_by_client_id(
                    &subscriber.topic_name,
                    &subscriber.client_id,
                );
            }
        }

        for (key, raw) in self.share_push.clone() {
            if raw.sub_list.is_empty() {
                self.share_push.remove(&key);
            }
        }
    }

    // share_push_thread
    pub fn share_push_thread_list(&self) -> DashMap<String, SubPushThreadData> {
        self.share_push_thread.clone()
    }

    pub fn share_push_thread_len(&self) -> u64 {
        self.share_push_thread.len() as u64
    }

    pub fn add_leader_push_thread(&self, share_leader_key: String, data: SubPushThreadData) {
        self.share_push_thread.insert(share_leader_key, data);
    }

    pub fn get_leader_push_thread(&self, share_leader_key: &str) -> Option<SubPushThreadData> {
        if let Some(data) = self.share_push_thread.get(share_leader_key) {
            return Some(data.clone());
        }
        None
    }

    pub fn remove_leader_push_thread(&self, share_leader_key: &str) {
        self.share_push_thread.remove(share_leader_key);
    }

    pub fn contain_leader_push_thread(&self, share_leader_key: &str) -> bool {
        self.share_push_thread.contains_key(share_leader_key)
    }

    pub fn update_subscribe_push_thread_info(
        &self,
        key: &str,
        success_record_num: u64,
        error_record_num: u64,
    ) {
        if let Some(mut thread) = self.share_push_thread.get_mut(key) {
            thread.last_run_time = now_second();

            if (success_record_num + error_record_num) > 0 {
                thread.push_success_record_num += success_record_num;
                thread.push_error_record_num += error_record_num;
                thread.last_push_time = now_second();
            }
        }
    }
}
