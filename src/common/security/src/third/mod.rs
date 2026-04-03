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

#![allow(clippy::result_large_err)]

use common_base::error::common::CommonError;
use metadata_struct::mqtt::auth::storage::StorageConfig;
use std::str::FromStr;
use std::sync::Arc;

pub mod http;
pub mod meta;
pub mod mongodb;
pub mod mysql;
pub mod postgresql;
pub mod redis;
pub mod storage_trait;
pub mod storage_type;
pub mod sync;
pub use storage_type::AuthDataStorageType;

use crate::third::storage_trait::AuthStorageAdapter;

pub fn build_storage_driver(
    storage_config: &StorageConfig,
) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, CommonError> {
    let storage_type = AuthDataStorageType::from_str(&storage_config.storage_type)
        .map_err(|_| CommonError::UnavailableStorageType)?;

    match storage_type {
        AuthDataStorageType::Meta => {
            let driver = meta::MetaServiceAuthStorageAdapter::new();
            Ok(Arc::new(driver))
        }
        AuthDataStorageType::Mysql => {
            if let Some(mysql_config) = &storage_config.mysql_config {
                let driver = mysql::MySQLAuthStorageAdapter::new(mysql_config.clone())?;
                Ok(Arc::new(driver))
            } else {
                Err(CommonError::CommonError(
                    "Mysql config not found".to_string(),
                ))
            }
        }
        AuthDataStorageType::Postgresql => {
            if let Some(postgres_config) = &storage_config.postgres_config {
                let driver =
                    postgresql::PostgresqlAuthStorageAdapter::new(postgres_config.clone())?;
                Ok(Arc::new(driver))
            } else {
                Err(CommonError::CommonError(
                    "Postgres config not found".to_string(),
                ))
            }
        }
        AuthDataStorageType::Redis => {
            if let Some(redis_config) = &storage_config.redis_config {
                let driver = redis::RedisAuthStorageAdapter::new(redis_config.clone())?;
                Ok(Arc::new(driver))
            } else {
                Err(CommonError::CommonError(
                    "Redis config not found".to_string(),
                ))
            }
        }
        AuthDataStorageType::Mongodb => {
            if let Some(mongodb_config) = &storage_config.mongodb_config {
                let driver = mongodb::MongoDBAuthStorageAdapter::new(mongodb_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(CommonError::CommonError(
                    "MongoDB config not found".to_string(),
                ))
            }
        }
        AuthDataStorageType::Http => {
            if let Some(http_config) = &storage_config.http_config {
                let driver = http::HttpAuthStorageAdapter::new(http_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(CommonError::HttpConfigNotFound)
            }
        }
    }
}
