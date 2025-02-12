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

use crate::storage::topic::TopicStorage;
use crate::{
    security::AuthDriver, storage::cluster::ClusterStorage,
    subscribe::subscribe_manager::SubscribeManager,
};
use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use log::error;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MqttTopic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::mqtt::{cluster::MqttClusterDynamicConfig, session::MqttSession};
use protocol::broker_mqtt::broker_mqtt_inner::{
    MqttBrokerUpdateCacheActionType, MqttBrokerUpdateCacheResourceType, UpdateMqttCacheRequest,
};
use std::sync::Arc;

use super::{cache::CacheManager, sub_exclusive::remove_exclusive_subscribe_by_path};

pub async fn load_metadata_cache(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    auth_driver: &Arc<AuthDriver>,
) {
    let conf = broker_mqtt_conf();
    // load cluster config
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let cluster = match cluster_storage.get_cluster_config(&conf.cluster_name).await {
        Ok(Some(cluster)) => cluster,
        Ok(None) => MqttClusterDynamicConfig::new(),
        Err(e) => {
            panic!(
                "Failed to load the cluster configuration with error message:{}",
                e
            );
        }
    };
    cache_manager.set_cluster_info(cluster);

    // load all topic
    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_list = match topic_storage.all().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the topic list with error message:{}", e);
        }
    };

    for (_, topic) in topic_list {
        cache_manager.add_topic(&topic.topic_name, &topic);
    }

    // load all user
    let user_list = match auth_driver.read_all_user().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the user list with error message:{}", e);
        }
    };

    for (_, user) in user_list {
        cache_manager.add_user(user);
    }

    // load all acl
    let acl_list = match auth_driver.read_all_acl().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the acl list with error message:{}", e);
        }
    };
    for acl in acl_list {
        cache_manager.add_acl(acl);
    }

    // load all blacklist
    let blacklist_list = match auth_driver.read_all_blacklist().await {
        Ok(list) => list,
        Err(e) => {
            panic!("Failed to load the blacklist list with error message:{}", e);
        }
    };
    for blacklist in blacklist_list {
        cache_manager.add_blacklist(blacklist);
    }

    // load All topic_rewrite rule
    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_rewrite_rules = match topic_storage.all_topic_rewrite_rule().await {
        Ok(list) => list,
        Err(e) => {
            panic!(
                "Failed to load the topic_rewrite_rule list with error message:{}",
                e
            );
        }
    };
    for topic_rewrite_rule in topic_rewrite_rules {
        cache_manager.add_topic_rewrite_rule(topic_rewrite_rule);
    }
}

pub async fn update_cache_metadata(
    cache_manager: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    request: UpdateMqttCacheRequest,
) {
    match request.resource_type() {
        MqttBrokerUpdateCacheResourceType::Session => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttSession>(&request.data) {
                    Ok(session) => {
                        cache_manager.add_session(session.client_id.clone(), session);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttSession>(&request.data) {
                    Ok(session) => {
                        cache_manager.remove_session(&session.client_id);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::User => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttUser>(&request.data) {
                    Ok(user) => {
                        cache_manager.add_user(user);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttUser>(&request.data) {
                    Ok(user) => {
                        cache_manager.del_user(user.username);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::Subscribe => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttSubscribe>(&request.data) {
                    Ok(subscribe) => {
                        subscribe_manager.add_subscribe(subscribe);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttSubscribe>(&request.data) {
                    Ok(subscribe) => {
                        if let Some(_sub) =
                            subscribe_manager.get_subscribe(&subscribe.client_id, &subscribe.path)
                        {
                            remove_exclusive_subscribe_by_path(subscribe_manager, &subscribe.path);
                        }
                        subscribe_manager.remove_subscribe(&subscribe.client_id, &subscribe.path)
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
        MqttBrokerUpdateCacheResourceType::Topic => match request.action_type() {
            MqttBrokerUpdateCacheActionType::Set => {
                match serde_json::from_str::<MqttTopic>(&request.data) {
                    Ok(topic) => {
                        cache_manager.add_topic(&topic.topic_name, &topic);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MqttBrokerUpdateCacheActionType::Delete => {
                match serde_json::from_str::<MqttTopic>(&request.data) {
                    Ok(topic) => {
                        cache_manager.delete_topic(&topic.topic_name, &topic);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
    }
}
