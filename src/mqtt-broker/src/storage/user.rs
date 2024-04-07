use super::{
    all::AllInfoStorage,
    keys::{all_user_key, user_key},
};
use crate::metadata::user::User;
use common_base::errors::RobustMQError;
use std::collections::HashMap;
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

pub struct UserStorage {
    storage_adapter: PlacementStorageAdapter,
    all_info_storage: AllInfoStorage,
}

impl UserStorage {
    pub fn new() -> UserStorage {
        let storage_adapter = PlacementStorageAdapter::new();
        let all_info_storage = AllInfoStorage::new(all_user_key());
        return UserStorage {
            storage_adapter,
            all_info_storage,
        };
    }

    // Saving user information
    pub fn save_user(&self, user_info: User) -> Result<(), RobustMQError> {
        let username = user_info.username.clone();
        let key = user_key(username.clone());
        match serde_json::to_string(&user_info) {
            Ok(data) => {
                match self.all_info_storage.add_info_for_all(username) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
                return self.storage_adapter.kv_set(key, data);
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Getting user information
    pub fn get_user(&self, username: String) -> Result<User, RobustMQError> {
        let key = user_key(username);
        match self.storage_adapter.kv_get(key) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(
                        e.to_string(),
                    ))
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Getting a list of users
    pub fn user_list(&self) -> Result<HashMap<String, User>, RobustMQError> {
        match self.all_info_storage.get_all() {
            Ok(data) => {
                let mut list = HashMap::new();
                for username in data {
                    match self.get_user(username.clone()) {
                        Ok(user) => {
                            list.insert(username, user);
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
