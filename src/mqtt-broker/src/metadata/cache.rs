use super::{cluster::Cluster, session::Session, topic::Topic, user::User};
use crate::storage::{cluster::ClusterStorage, topic::TopicStorage, user::UserStorage};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
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
pub struct MetadataCache<T> {
    pub cluster_info: Cluster,
    pub user_info: DashMap<String, User>,
    pub session_info: DashMap<String, Session>,
    pub topic_info: DashMap<String, Topic>,
    pub topic_id_name: DashMap<String, String>,
    pub connect_id_info: DashMap<u64, String>,
    pub login_info: DashMap<u64, bool>,
    pub metadata_storage_adapter: Arc<T>,
}

impl<T> MetadataCache<T>
where
    T: StorageAdapter,
{
    pub fn new(metadata_storage_adapter: Arc<T>) -> Self {
        let cache = MetadataCache {
            cluster_info: Cluster::default(),
            user_info: DashMap::with_capacity(256),
            session_info: DashMap::with_capacity(256),
            topic_info: DashMap::with_capacity(256),
            topic_id_name: DashMap::with_capacity(256),
            connect_id_info: DashMap::with_capacity(256),
            login_info: DashMap::with_capacity(256),
            metadata_storage_adapter,
        };
        return cache;
    }

    pub async fn load_cache(&mut self) {
        // load cluster config
        let cluster_storage = ClusterStorage::new(self.metadata_storage_adapter.clone());
        self.cluster_info = match cluster_storage.get_cluster_config().await {
            Ok(Some(cluster)) => cluster,
            Ok(None) => Cluster::new(),
            Err(e) => {
                panic!(
                    "Failed to load the cluster configuration with error message:{}",
                    e.to_string()
                );
            }
        };

        // load all user
        let user_storage = UserStorage::new(self.metadata_storage_adapter.clone());
        self.user_info = match user_storage.user_list().await {
            Ok(list) => list,
            Err(e) => {
                panic!(
                    "Failed to load the user list with error message:{}",
                    e.to_string()
                );
            }
        };

        // load topic info
        let topic_storage = TopicStorage::new(self.metadata_storage_adapter.clone());
        self.topic_info = match topic_storage.topic_list().await {
            Ok(list) => list,
            Err(e) => {
                panic!(
                    "Failed to load the topic list with error message:{}",
                    e.to_string()
                );
            }
        };

        for (topic_name, topic) in self.topic_info.clone() {
            self.topic_id_name.insert(topic.topic_id, topic_name);
        }
    }

    pub fn apply(&self, data: String) {
        let data: MetadataChangeData = serde_json::from_str(&data).unwrap();
        match data.data_type {
            MetadataCacheType::User => match data.action {
                MetadataCacheAction::Set => {
                    let user: User = serde_json::from_str(&data.value).unwrap();
                    self.set_user(user);
                }
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

    pub fn set_cluster_info(&self, cluster: Cluster) {
        self.cluster_info = cluster;
    }

    pub fn set_user(&self, user: User) {
        self.user_info.insert(user.username.clone(), user);
    }

    pub fn del_user(&self, value: String) {
        let data: User = serde_json::from_str(&value).unwrap();
        self.user_info.remove(&data.username);
    }

    pub fn set_session(&self, client_id: String, session: Session) {
        self.session_info.insert(client_id, session);
    }

    pub fn set_client_id(&self, connect_id: u64, client_id: String) {
        self.connect_id_info.insert(connect_id, client_id);
    }

    pub fn set_topic(&self, topic_name: &String, topic: &Topic) {
        let t = topic.clone();
        self.topic_info.insert(topic_name.clone(), t.clone());
        self.topic_id_name.insert(t.topic_id, topic_name.clone());
    }

    pub fn login_success(&self, connect_id: u64) {
        self.login_info.insert(connect_id, true);
    }

    pub fn is_login(&self, connect_id: u64) -> bool {
        return self.login_info.contains_key(&connect_id);
    }

    pub fn topic_exists(&self, topic: &String) -> bool {
        return self.topic_info.contains_key(topic);
    }

    pub fn topic_name_by_id(&self, topic_id: String) -> Option<String> {
        if let Some(data) = self.topic_id_name.get(&topic_id) {
            return Some(data.clone());
        }
        return None;
    }

    pub fn get_topic_by_name(&self, topic_name: String) -> Option<Topic> {
        if let Some(topic) = self.topic_info.get(&topic_name) {
            return Some(topic.clone());
        }
        return None;
    }

    pub fn remove_connect_id(&self, connect_id: u64) {
        if let Some(client_id) = self.connect_id_info.get(&connect_id) {
            self.session_info.remove(&*client_id);
            self.login_info.remove(&connect_id);
            self.connect_id_info.remove(&connect_id);
        }
    }

}
