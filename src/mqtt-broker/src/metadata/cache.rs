use crate::storage::{cluster::ClusterStorage, topic::TopicStorage, user::UserStorage};

use super::{cluster::Cluster, session::Session, subscriber::Subscriber, topic::Topic, user::User};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use storage_adapter::memory::MemoryStorageAdapter;
#[derive(Clone, Serialize, Deserialize)]
pub enum MetadataCacheAction {
    Set,
    Del,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MetadataCacheType {
    Cluster,
    User,
    Topic,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MetadataChangeData {
    pub action: MetadataCacheAction,
    pub data_type: MetadataCacheType,
    pub value: String,
}

#[derive(Clone)]
pub struct MetadataCache {
    pub cluster_info: Cluster,
    pub user_info: HashMap<String, User>,
    pub session_info: HashMap<String, Session>,
    pub topic_info: HashMap<String, Topic>,
    pub subscriber_info: HashMap<String, Subscriber>,
    pub connect_id_info: HashMap<u64, String>,
    pub login_info: HashMap<u64, bool>,
    pub storage_adapter: Arc<MemoryStorageAdapter>,
}

impl MetadataCache {
    pub fn new(storage_adapter: Arc<MemoryStorageAdapter>) -> Self {
        let cache = MetadataCache {
            cluster_info: Cluster::default(),
            user_info: HashMap::new(),
            session_info: HashMap::new(),
            topic_info: HashMap::new(),
            subscriber_info: HashMap::new(),
            connect_id_info: HashMap::new(),
            login_info: HashMap::new(),
            storage_adapter,
        };
        return cache;
    }

    pub async fn load_cache(&mut self) {
        // load cluster config
        let cluster_storage = ClusterStorage::new(self.storage_adapter.clone());
        self.cluster_info = match cluster_storage.get_cluster_config().await {
            Ok(cluster) => {
                if let Some(data) = cluster {
                    data
                } else {
                    Cluster::new()
                }
            }
            Err(e) => {
                panic!(
                    "Failed to load the cluster configuration with error message:{}",
                    e.to_string()
                );
            }
        };

        // load all user
        let user_storage = UserStorage::new(self.storage_adapter.clone());
        self.user_info = match user_storage.user_list().await {
            Ok(list) => list,
            Err(e) => {
                panic!(
                    "Failed to load the user list with error message:{}",
                    e.to_string()
                );
            }
        };

        // Not all session information is loaded at startup, only when the client is connected,
        // if the clean session is set, it will check whether the session exists and then update the local cache.
        self.session_info = HashMap::new();

        // load topic info
        let topic_storage = TopicStorage::new(self.storage_adapter.clone());
        self.topic_info = match topic_storage.topic_list().await {
            Ok(list) => list,
            Err(e) => {
                panic!(
                    "Failed to load the topic list with error message:{}",
                    e.to_string()
                );
            }
        };

        // subscriber, connect, and login are connection-related and don't need to be persisted, so they don't need to be loaded.
        self.subscriber_info = HashMap::new();
        self.connect_id_info = HashMap::new();
        self.login_info = HashMap::new();
    }

    pub fn apply(&mut self, data: String) {
        let data: MetadataChangeData = serde_json::from_str(&data).unwrap();
        match data.data_type {
            MetadataCacheType::User => match data.action {
                MetadataCacheAction::Set => self.set_user(data.value),
                MetadataCacheAction::Del => self.del_user(data.value),
            },
            MetadataCacheType::Topic => match data.action {
                MetadataCacheAction::Set => {}
                MetadataCacheAction::Del => {}
            },
            MetadataCacheType::Cluster => match data.action {
                MetadataCacheAction::Set => {}
                MetadataCacheAction::Del => {}
            },
        }
    }

    pub fn set_user(&mut self, value: String) {
        let data: User = serde_json::from_str(&value).unwrap();
        self.user_info.insert(data.username.clone(), data);
    }

    pub fn del_user(&mut self, value: String) {
        let data: User = serde_json::from_str(&value).unwrap();
        self.user_info.remove(&data.username);
    }

    pub fn set_session(&mut self, client_id: String, session: Session) {
        self.session_info.insert(client_id, session);
    }

    pub fn set_client_id(&mut self, connect_id: u64, client_id: String) {
        self.connect_id_info.insert(connect_id, client_id);
    }

    pub fn set_topic(&mut self, topic_name: &String, topic: &Topic) {
        self.topic_info.insert(topic_name.clone(), topic.clone());
    }

    pub fn login_success(&mut self, connect_id: u64) {
        self.login_info.insert(connect_id, true);
    }

    pub fn is_login(&self, connect_id: u64) -> bool {
        return self.login_info.contains_key(&connect_id);
    }

    pub fn topic_exists(&self, topic: &String) -> bool {
        return self.topic_info.contains_key(topic);
    }

    pub fn remove_connect_id(&mut self, connect_id: u64) {
        if let Some(client_id) = self.connect_id_info.get(&connect_id) {
            self.session_info.remove(client_id);
            self.login_info.remove(&connect_id);
            self.connect_id_info.remove(&connect_id);
        }
    }
}
