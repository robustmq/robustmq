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

use crate::core::error::MqttBrokerError;
use crate::security::storage::storage_trait::AuthStorageAdapter;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::security::StorageConfig;
use std::str::FromStr;
use std::sync::Arc;

pub mod http;
pub mod meta;
pub mod mysql;
pub mod postgresql;
pub mod redis;
pub mod storage_trait;
pub mod storage_type;
pub mod sync;
pub use storage_type::AuthDataStorageType;

pub fn build_storage_driver(
    client_pool: &Arc<ClientPool>,
    storage_config: &StorageConfig,
) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, MqttBrokerError> {
    let storage_type = AuthDataStorageType::from_str(&storage_config.storage_type)
        .map_err(|_| MqttBrokerError::UnavailableStorageType)?;

    match storage_type {
        AuthDataStorageType::Meta => {
            let driver = meta::MetaServiceAuthStorageAdapter::new(client_pool.clone());
            Ok(Arc::new(driver))
        }
        AuthDataStorageType::Mysql => {
            if let Some(mysql_config) = &storage_config.mysql_config {
                let driver = mysql::MySQLAuthStorageAdapter::new(mysql_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::CommonError(
                    "Mysql config not found".to_string(),
                ))
            }
        }
        AuthDataStorageType::Postgresql => {
            if let Some(postgres_config) = &storage_config.postgres_config {
                let driver = postgresql::PostgresqlAuthStorageAdapter::new(postgres_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::CommonError(
                    "Postgres config not found".to_string(),
                ))
            }
        }
        AuthDataStorageType::Redis => {
            if let Some(redis_config) = &storage_config.redis_config {
                let driver = redis::RedisAuthStorageAdapter::new(redis_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::CommonError(
                    "Redis config not found".to_string(),
                ))
            }
        }
        AuthDataStorageType::Http => {
            if let Some(http_config) = &storage_config.http_config {
                let driver = http::HttpAuthStorageAdapter::new(http_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::HttpConfigNotFound)
            }
        }
    }
}
