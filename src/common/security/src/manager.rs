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
use dashmap::DashMap;
use metadata_struct::mqtt::auth::authn_config::AuthnConfig;
use metadata_struct::mqtt::auth::authn_config::LoginAuthEnum;
use metadata_struct::mqtt::auth::password::PasswordBasedConfig;
use metadata_struct::mqtt::auth::storage::StorageConfig;
use std::sync::Arc;
use tracing::warn;

pub type ArcAuthStorageAdapter = Arc<dyn AuthStorageAdapter + Send + Sync>;

#[derive(Clone, Default)]
pub struct SecurityManager {
    storage_drivers: Arc<DashMap<String, ArcAuthStorageAdapter>>,
    pub security_metadata: SecurityMetadata,
}

impl SecurityManager {
    pub fn new() -> SecurityManager {
        SecurityManager {
            storage_drivers: Arc::new(DashMap::new()),
            security_metadata: SecurityMetadata::new(),
        }
    }

    pub fn authn_list_with_default(&self) -> Vec<(String, AuthnConfig)> {
        let mut authn_list = self.security_metadata.authn_list();
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

    pub async fn drivers_list(&self) -> Result<Vec<ArcAuthStorageAdapter>, CommonError> {
        let mut drivers = Vec::new();
        for (authn_id, authn) in self.authn_list_with_default() {
            match authn.config {
                LoginAuthEnum::PasswordBased(config) => {
                    if let Some(driver) = self
                        .get_or_build_storage_driver(&authn_id, &config.storage_config)
                        .await?
                    {
                        drivers.push(driver);
                    }
                }
                _ => {
                    warn!(authn_id = %authn_id, authn_type = %authn.authn_type, "Unsupported authn type, skipped");
                }
            }
        }
        Ok(drivers)
    }

    async fn get_or_build_storage_driver(
        &self,
        authn_id: &str,
        storage_config: &StorageConfig,
    ) -> Result<Option<ArcAuthStorageAdapter>, CommonError> {
        if let Some(driver) = self.storage_drivers.get(authn_id) {
            return Ok(Some(driver.clone()));
        }

        if let Some(driver) = build_storage_driver(storage_config).await? {
            self.storage_drivers
                .insert(authn_id.to_string(), driver.clone());
        }

        Ok(None)
    }
}
