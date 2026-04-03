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

use crate::metadata::SecurityMetadata;
use crate::third::build_storage_driver;
use crate::third::storage_trait::AuthStorageAdapter;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::now_millis;
use dashmap::DashMap;
use metadata_struct::auth::acl::SecurityAcl;
use metadata_struct::auth::blacklist::SecurityBlackList;
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::mqtt::auth::authn_config::AuthnConfig;
use metadata_struct::mqtt::auth::authn_config::LoginAuthEnum;
use metadata_struct::mqtt::auth::password::PasswordBasedConfig;
use metadata_struct::mqtt::auth::storage::StorageConfig;
use std::collections::HashMap;
use std::sync::Arc;

const STORAGE_DRIVER_REBUILD_MS: u128 = 5000;

#[derive(Clone)]
struct CachedStorageDriver {
    driver: Arc<dyn AuthStorageAdapter + Send + Sync>,
    build_at_ms: u128,
}

#[derive(Clone)]
pub struct SecurityManager {
    storage_drivers: Arc<DashMap<String, CachedStorageDriver>>,
    pub security_metadata: SecurityMetadata,
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityManager {
    pub fn new() -> SecurityManager {
        SecurityManager {
            storage_drivers: Arc::new(DashMap::new()),
            security_metadata: SecurityMetadata::new(),
        }
    }

    pub fn authn_list_with_default(&self) -> Vec<(String, AuthnConfig)> {
        let mut authn_list = self.security_metadata.get_authn();
        if authn_list.is_empty() {
            let default_authn = AuthnConfig {
                uid: "inner_default".to_string(),
                authn_type: "password_based".to_string(),
                config: LoginAuthEnum::PasswordBased(Box::new(PasswordBasedConfig {
                    storage_config: StorageConfig {
                        storage_type: "meta".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                create_at: 0,
            };
            authn_list.push((default_authn.uid.clone(), default_authn));
        }
        authn_list
    }

    pub fn get_or_build_storage_driver(
        &self,
        authn_id: &str,
        storage_config: &StorageConfig,
    ) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, CommonError> {
        let now = now_millis();
        if let Some(cached) = self.storage_drivers.get(authn_id) {
            if now.saturating_sub(cached.build_at_ms) <= STORAGE_DRIVER_REBUILD_MS {
                return Ok(cached.driver.clone());
            }
        }

        let driver = build_storage_driver(storage_config)?;
        self.storage_drivers.insert(
            authn_id.to_string(),
            CachedStorageDriver {
                driver: driver.clone(),
                build_at_ms: now,
            },
        );
        Ok(driver)
    }

    fn password_based_drivers(
        &self,
    ) -> Result<Vec<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>>, CommonError> {
        let mut drivers = Vec::new();
        for (authn_id, authn) in self.authn_list_with_default() {
            if let LoginAuthEnum::PasswordBased(config) = authn.config {
                let driver = self.get_or_build_storage_driver(&authn_id, &config.storage_config)?;
                drivers.push(driver);
            }
        }
        Ok(drivers)
    }

    // read all
    pub async fn read_all_user(&self) -> Result<HashMap<String, SecurityUser>, CommonError> {
        let mut results = HashMap::new();
        for driver in self.password_based_drivers()? {
            let list = driver.read_all_user().await?;
            for user in list.iter() {
                results.insert(user.username.clone(), user.clone());
            }
        }
        Ok(results)
    }

    pub async fn read_all_acl(&self) -> Result<Vec<SecurityAcl>, CommonError> {
        let mut results = Vec::new();
        for driver in self.password_based_drivers()? {
            let list = driver.read_all_acl().await?;
            for acl in list.iter() {
                results.push(acl.clone());
            }
        }
        Ok(results)
    }

    pub async fn read_all_blacklist(&self) -> Result<Vec<SecurityBlackList>, CommonError> {
        let mut results = Vec::new();
        for driver in self.password_based_drivers()? {
            let list = driver.read_all_blacklist().await?;
            for blacklist in list.iter() {
                results.push(blacklist.clone());
            }
        }
        Ok(results)
    }

    pub async fn update_user_cache(&self) -> ResultCommonError {
        let all_users: HashMap<String, SecurityUser> = self.read_all_user().await?;

        for user in all_users.values() {
            self.security_metadata.add_user(user.clone());
        }

        Ok(())
    }

    // ACL
    pub async fn update_acl_cache(&self) -> ResultCommonError {
        let all_acls: Vec<SecurityAcl> = self.read_all_acl().await?;

        for acl in all_acls.iter() {
            self.security_metadata.add_acl(acl.to_owned());
        }

        Ok(())
    }

    // BlackList
    pub async fn update_blacklist_cache(&self) -> ResultCommonError {
        let all_blacklist = self.read_all_blacklist().await?;

        for acl in all_blacklist.iter() {
            self.security_metadata.add_blacklist(acl.to_owned());
        }

        Ok(())
    }
}
