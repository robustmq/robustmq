use super::{
    all::AllInfoStorage,
    keys::{all_user_key, user_key},
};
use crate::metadata::user::User;
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};
use storage_adapter::{record::Record, storage::StorageAdapter};

pub struct UserStorage<T> {
    storage_adapter: Arc<T>,
    all_info_storage: AllInfoStorage<T>,
}

impl<T> UserStorage<T>
where
    T: StorageAdapter,
{
    pub fn new(storage_adapter: Arc<T>) -> Self {
        let all_info_storage = AllInfoStorage::new(all_user_key(), storage_adapter.clone());
        return UserStorage {
            storage_adapter,
            all_info_storage,
        };
    }

    // Saving user information
    pub async fn save_user(&self, user_info: User) -> Result<(), RobustMQError> {
        let username = user_info.username.clone();
        let key = user_key(username.clone());
        match serde_json::to_vec(&user_info) {
            Ok(data) => {
                match self.all_info_storage.add_info_for_all(username).await {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
                return self.storage_adapter.set(key, Record::build_b(data)).await;
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save user config error, error messsage:{}",
                    e.to_string()
                )))
            }
        }
    }

    // Getting user information
    pub async fn get_user(&self, username: String) -> Result<Option<User>, RobustMQError> {
        let key = user_key(username);
        match self.storage_adapter.get(key).await {
            Ok(Some(data)) => match serde_json::from_slice(&data.data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(format!(
                        "get user config error, error messsage:{}",
                        e.to_string()
                    )))
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Getting a list of users
    pub async fn user_list(&self) -> Result<DashMap<String, User>, RobustMQError> {
        match self.all_info_storage.get_all().await {
            Ok(data) => {
                let list = DashMap::with_capacity(256);
                for username in data {
                    match self.get_user(username.clone()).await {
                        Ok(user) => {
                            if let Some(t) = user {
                                list.insert(username, t);
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                return Ok(list);
            }
            Err(e) => return Err(e),
        }
    }
}
