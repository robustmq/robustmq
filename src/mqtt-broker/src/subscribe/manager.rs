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

use crate::{
    handler::sub_exclusive::is_exclusive_sub,
    subscribe::{buckets::BucketsManager, common::Subscriber},
};
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ShareLeaderSubscribeData {
    pub path: String,

    // group
    pub group_name: String,
    pub sub_path: String,

    // topic
    pub topic_name: String,

    // (client_id, subscriber)
    pub sub_list: DashMap<String, Subscriber>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TopicSubscribeInfo {
    pub client_id: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct TemporaryNotPushClient {
    pub client_id: String,
    pub last_check_time: u64,
}

#[derive(Clone, Default)]
pub struct SubscribeManager {
    //(client_id_path: MqttSubscribe)
    pub subscribe_list: DashMap<String, MqttSubscribe>,

    // directly sub
    pub directly_push: BucketsManager,

    // share sub
    pub share_push: BucketsManager,

    //(topic_name, Vec<TopicSubscribeInfo>)
    pub topic_subscribes: DashMap<String, Vec<TopicSubscribeInfo>>,

    //(client_id, TemporaryNotPushClient)
    pub not_push_client: DashMap<String, TemporaryNotPushClient>,
}

impl SubscribeManager {
    pub fn new() -> Self {
        SubscribeManager {
            subscribe_list: DashMap::with_capacity(8),
            topic_subscribes: DashMap::with_capacity(8),
            not_push_client: DashMap::with_capacity(8),
            directly_push: BucketsManager::new(10000),
            share_push: BucketsManager::new(10000),
        }
    }

    // subscribe_list
    pub fn add_subscribe(&self, subscribe: &MqttSubscribe) {
        let key = self.subscribe_key(&subscribe.client_id, &subscribe.path);
        self.subscribe_list.insert(key, subscribe.clone());
    }

    pub fn get_subscribe(&self, client_id: &str, path: &str) -> Option<MqttSubscribe> {
        let key = self.subscribe_key(client_id, path);
        if let Some(da) = self.subscribe_list.get(&key) {
            return Some(da.clone());
        }
        None
    }

    // directly && share
    pub async fn add_directly_sub(&self, topic: &str, subscriber: &Subscriber) {
        self.add_topic_subscribe(topic, &subscriber.client_id, &subscriber.sub_path);
        self.directly_push.add(subscriber).await;
    }

    pub fn add_share_sub(&self, topic: &str, subscriber: &Subscriber) {
        self.add_topic_subscribe(topic, &subscriber.client_id, &subscriber.sub_path);
        // todo
    }

    // remove
    pub async fn remove_by_client_id(&self, client_id: &str) {
        self.subscribe_list
            .retain(|_, subscribe| subscribe.client_id != *client_id);

        for mut list in self.topic_subscribes.iter_mut() {
            list.retain(|x| x.client_id != *client_id);
        }

        self.not_push_client.remove(client_id);

        self.directly_push.remove_by_client_id(client_id).await;
    }

    pub async fn remove_by_sub(&self, client_id: &str, sub_path: &str) {
        // subscribe list
        let key = self.subscribe_key(client_id, sub_path);
        self.subscribe_list.remove(&key);

        // topic subscribe
        for mut list in self.topic_subscribes.iter_mut() {
            list.retain(|x| x.path != *sub_path && x.client_id != *client_id);
        }

        // remove directly sub
        self.directly_push.remove_by_sub(client_id, sub_path).await;
    }

    pub fn remove_by_topic(&self, _topic_name: &str) {
        // todo
    }

    // Not push client
    pub fn add_not_push_client(&self, client_id: &str) {
        self.not_push_client.insert(
            client_id.to_string(),
            TemporaryNotPushClient {
                client_id: client_id.to_string(),
                last_check_time: now_second(),
            },
        );
    }

    pub fn update_not_push_client(&self, client_id: &str) {
        if let Some(mut raw) = self.not_push_client.get_mut(client_id) {
            raw.last_check_time = now_second();
        }
    }

    // topic subscribe
    pub fn add_topic_subscribe(&self, topic_name: &str, client_id: &str, path: &str) {
        if let Some(mut list) = self.topic_subscribes.get_mut(topic_name) {
            if !list
                .iter()
                .any(|info| info.client_id == client_id && info.path == path)
            {
                list.push(TopicSubscribeInfo {
                    client_id: client_id.to_owned(),
                    path: path.to_owned(),
                });
            }
        } else {
            self.topic_subscribes.insert(
                topic_name.to_owned(),
                vec![TopicSubscribeInfo {
                    client_id: client_id.to_owned(),
                    path: path.to_owned(),
                }],
            );
        }
    }

    pub fn is_exclusive_subscribe(&self, topic_name: &str) -> bool {
        if let Some(list) = self.topic_subscribes.get(topic_name) {
            for raw in list.iter() {
                if is_exclusive_sub(&raw.path) {
                    return true;
                }
            }
        }
        false
    }

    fn subscribe_key(&self, client_id: &str, path: &str) -> String {
        format!("{client_id}_{path}")
    }
}

#[cfg(test)]
mod tests {}
