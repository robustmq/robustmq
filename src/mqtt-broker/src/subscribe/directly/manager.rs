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

use crate::subscribe::{buckets::BucketsManager, common::Subscriber};
use dashmap::DashMap;

#[derive(Clone, Default)]
pub struct DirectlySubManager {
    // (client_id_sub_path_topic_name, Subscriber)
    pub directly_client_id_push: BucketsManager,

    // (client_id_sub_path_topic_name, Subscriber)
    pub directly_topic_push: DashMap<String, BucketsManager>,
}

impl DirectlySubManager {
    pub fn new() -> Self {
        DirectlySubManager {
            directly_client_id_push: BucketsManager::new(10000),
            directly_topic_push: DashMap::with_capacity(8),
        }
    }

    pub fn add_sub(&self, subscriber: &Subscriber) {
        self.add_client_id_sub(subscriber.clone());
        self.add_topic_sub(subscriber);
    }

    fn add_client_id_sub(&self, subscriber: Subscriber) {
        self.directly_client_id_push.add(subscriber);
    }

    fn add_topic_sub(&self, subscriber: &Subscriber) {
        if let Some(data) = self.directly_topic_push.get(&subscriber.topic_name) {
            data.add(subscriber.clone());
        } else {
            let push_group_manager = BucketsManager::new(10000);
            push_group_manager.add(subscriber.clone());
            self.directly_topic_push
                .insert(subscriber.topic_name.clone(), push_group_manager);
        }
    }

    pub fn len(&self) -> u32 {
        0
    }

    pub fn client_key(&self, client_id: &str, topic_name: &str) -> String {
        format!("{client_id}_{topic_name}")
    }
}
