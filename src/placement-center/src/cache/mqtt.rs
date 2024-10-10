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

use std::sync::Arc;

use dashmap::DashMap;
use metadata_struct::mqtt::topic::MqttTopic;
use metadata_struct::mqtt::user::MqttUser;
use protocol::placement_center::generate::common::ClusterType;

use super::placement::PlacementCacheManager;
use crate::controller::mqtt::session_expire::ExpireLastWill;
use crate::storage::mqtt::topic::MqttTopicStorage;
use crate::storage::mqtt::user::MqttUserStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct MqttCacheManager {
    pub topic_list: DashMap<String, DashMap<String, MqttTopic>>,
    pub user_list: DashMap<String, DashMap<String, MqttUser>>,
    pub expire_last_wills: DashMap<String, DashMap<String, ExpireLastWill>>,
}

impl MqttCacheManager {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cache: Arc<PlacementCacheManager>,
    ) -> MqttCacheManager {
        let cache = MqttCacheManager {
            topic_list: DashMap::with_capacity(8),
            user_list: DashMap::with_capacity(8),
            expire_last_wills: DashMap::with_capacity(8),
        };
        cache.load_cache(rocksdb_engine_handler, placement_cache);
        cache
    }

    pub fn add_topic(&self, cluster_name: &String, topic: MqttTopic) {
        if let Some(data) = self.topic_list.get_mut(cluster_name) {
            data.insert(topic.topic_name.clone(), topic);
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(topic.topic_name.clone(), topic);
            self.topic_list.insert(cluster_name.clone(), data);
        }
    }

    #[allow(dead_code)]
    pub fn remove_topic(&self, cluster_name: &String, topic_name: &String) {
        if let Some(data) = self.topic_list.get_mut(cluster_name) {
            data.remove(topic_name);
        }
    }

    pub fn add_user(&self, cluster_name: &String, user: MqttUser) {
        if let Some(data) = self.user_list.get_mut(cluster_name) {
            data.insert(user.username.clone(), user);
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(user.username.clone(), user);
            self.user_list.insert(cluster_name.clone(), data);
        }
    }

    pub fn add_expire_last_will(&self, expire_last_will: ExpireLastWill) {
        if let Some(data) = self
            .expire_last_wills
            .get_mut(&expire_last_will.cluster_name)
        {
            data.insert(expire_last_will.client_id.clone(), expire_last_will);
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(expire_last_will.client_id.clone(), expire_last_will.clone());
            self.expire_last_wills
                .insert(expire_last_will.cluster_name.clone(), data);
        }
    }

    pub fn remove_expire_last_will(&self, cluster_name: &String, client_id: &String) {
        if let Some(data) = self.expire_last_wills.get_mut(cluster_name) {
            data.remove(client_id);
        }
    }

    pub fn load_cache(
        &self,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cache: Arc<PlacementCacheManager>,
    ) {
        for (_, cluster) in placement_cache.cluster_list.clone() {
            if cluster.cluster_type == *ClusterType::MqttBrokerServer.as_str_name() {
                let topic = MqttTopicStorage::new(rocksdb_engine_handler.clone());
                match topic.list(&cluster.cluster_name) {
                    Ok(data) => {
                        for topic in data {
                            self.add_topic(&cluster.cluster_name, topic);
                        }
                    }
                    Err(e) => {
                        panic!("{}", e.to_string())
                    }
                }

                let user = MqttUserStorage::new(rocksdb_engine_handler.clone());
                match user.list(&cluster.cluster_name) {
                    Ok(data) => {
                        for user in data {
                            self.add_user(&cluster.cluster_name, user);
                        }
                    }
                    Err(e) => {
                        panic!("{}", e.to_string())
                    }
                }
            }
        }
    }
}
