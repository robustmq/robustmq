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

use super::Authentication;
use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::security::storage::storage_trait::AuthStorageAdapter;
use axum::async_trait;
use bcrypt;
use common_config::security::PasswordConfig;
use hex;
use hmac::Hmac;
use md5;
use pbkdf2::pbkdf2;
use sha1::{Digest as Sha1Digest, Sha1};
#[allow(unused_imports)]
use sha2::{Digest as Sha2Digest, Sha256, Sha512};
use std::sync::Arc;

pub struct PostgreSQL {
    username: String,
    password: String,
    password_config: PasswordConfig,
    cache_manager: Arc<MQTTCacheManager>,
}

impl PostgreSQL {
    pub fn new(
        username: String,
        password: String,
        password_config: PasswordConfig,
        cache_manager: Arc<MQTTCacheManager>,
    ) -> Self {
        PostgreSQL {
            username,
            password,
            password_config,
            cache_manager,
        }
    }

    /// verify password
    fn verify_password(
        &self,
        stored_hash: &str,
        input_password: &str,
        salt: &str,
    ) -> Result<bool, MqttBrokerError> {
        match self.password_config.algorithm.as_str() {
            "plain" => Ok(stored_hash == input_password),
            "md5" => self.verify_md5(stored_hash, input_password, salt),
            "sha" | "sha1" => self.verify_sha1(stored_hash, input_password, salt),
            "sha256" => self.verify_sha256(stored_hash, input_password, salt),
            "sha512" => self.verify_sha512(stored_hash, input_password, salt),
            "bcrypt" => self.verify_bcrypt(stored_hash, input_password),
            "pbkdf2" => self.verify_pbkdf2(stored_hash, input_password, salt),
            _ => Err(MqttBrokerError::UnsupportedHashAlgorithm(
                self.password_config.algorithm.clone(),
            )),
        }
    }

    /// prepare password with salt
    fn prepare_password_with_salt(&self, password: &str, salt: &str) -> String {
        match self.password_config.salt_position.as_deref() {
            Some("prefix") => format!("{}{}", salt, password),
            Some("suffix") => format!("{}{}", password, salt),
            _ => password.to_string(), // disable or None
        }
    }

    /// verify MD5
    fn verify_md5(
        &self,
        stored_hash: &str,
        input_password: &str,
        salt: &str,
    ) -> Result<bool, MqttBrokerError> {
        let password_with_salt = self.prepare_password_with_salt(input_password, salt);
        let hash = format!("{:x}", md5::compute(password_with_salt.as_bytes()));
        Ok(hash == stored_hash)
    }

    /// verify SHA1
    fn verify_sha1(
        &self,
        stored_hash: &str,
        input_password: &str,
        salt: &str,
    ) -> Result<bool, MqttBrokerError> {
        let password_with_salt = self.prepare_password_with_salt(input_password, salt);
        let mut hasher = Sha1::new();
        hasher.update(password_with_salt.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash == stored_hash)
    }

    /// verify SHA256
    fn verify_sha256(
        &self,
        stored_hash: &str,
        input_password: &str,
        salt: &str,
    ) -> Result<bool, MqttBrokerError> {
        let password_with_salt = self.prepare_password_with_salt(input_password, salt);
        let mut hasher = Sha256::new();
        hasher.update(password_with_salt.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash == stored_hash)
    }

    /// verify SHA512
    fn verify_sha512(
        &self,
        stored_hash: &str,
        input_password: &str,
        salt: &str,
    ) -> Result<bool, MqttBrokerError> {
        let password_with_salt = self.prepare_password_with_salt(input_password, salt);
        let mut hasher = Sha512::new();
        hasher.update(password_with_salt.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash == stored_hash)
    }

    /// verify bcrypt
    fn verify_bcrypt(
        &self,
        stored_hash: &str,
        input_password: &str,
    ) -> Result<bool, MqttBrokerError> {
        bcrypt::verify(input_password, stored_hash)
            .map_err(|e| MqttBrokerError::PasswordVerificationError(e.to_string()))
    }

    /// verify PBKDF2
    #[allow(unused_must_use)]
    fn verify_pbkdf2(
        &self,
        stored_hash: &str,
        input_password: &str,
        salt: &str,
    ) -> Result<bool, MqttBrokerError> {
        let iterations = self.password_config.iterations.unwrap_or(4096);
        let dk_length = self.password_config.dk_length.unwrap_or(32) as usize;

        // use specified MAC function, default is SHA256
        let mac_fun = self.password_config.mac_fun.as_deref().unwrap_or("sha256");

        match mac_fun {
            "sha256" => {
                let mut derived_key = vec![0u8; dk_length];
                pbkdf2::<Hmac<Sha256>>(
                    input_password.as_bytes(),
                    salt.as_bytes(),
                    iterations,
                    &mut derived_key,
                );
                let computed_hash = hex::encode(derived_key);
                Ok(computed_hash == stored_hash)
            }
            "sha1" => {
                let mut derived_key = vec![0u8; dk_length];
                pbkdf2::<Hmac<Sha1>>(
                    input_password.as_bytes(),
                    salt.as_bytes(),
                    iterations,
                    &mut derived_key,
                );
                let computed_hash = hex::encode(derived_key);
                Ok(computed_hash == stored_hash)
            }
            "sha512" => {
                let mut derived_key = vec![0u8; dk_length];
                pbkdf2::<Hmac<Sha512>>(
                    input_password.as_bytes(),
                    salt.as_bytes(),
                    iterations,
                    &mut derived_key,
                );
                let computed_hash = hex::encode(derived_key);
                Ok(computed_hash == stored_hash)
            }
            _ => Err(MqttBrokerError::UnsupportedMacFunction(mac_fun.to_string())),
        }
    }
}

/// PostgreSQL authentication check entry function
pub async fn postgresql_check_login(
    driver: &Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
    cache_manager: &Arc<MQTTCacheManager>,
    password_config: &PasswordConfig,
    username: &str,
    password: &str,
) -> Result<bool, MqttBrokerError> {
    let postgresql_auth = PostgreSQL::new(
        username.to_owned(),
        password.to_owned(),
        password_config.clone(),
        cache_manager.clone(),
    );

    match postgresql_auth.apply().await {
        Ok(flag) => {
            if flag {
                return Ok(true);
            }
        }
        Err(e) => {
            // if user does not exist, try to get user information from storage layer
            if e.to_string() == MqttBrokerError::UserDoesNotExist.to_string() {
                return try_get_check_user_by_driver(
                    driver,
                    cache_manager,
                    password_config,
                    username,
                    password,
                )
                .await;
            }
            return Err(e);
        }
    }
    Ok(false)
}

/// try to get user from storage driver and verify
async fn try_get_check_user_by_driver(
    driver: &Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
    cache_manager: &Arc<MQTTCacheManager>,
    password_config: &PasswordConfig,
    username: &str,
    password: &str,
) -> Result<bool, MqttBrokerError> {
    if let Some(user) = driver.get_user(username.to_owned()).await? {
        cache_manager.add_user(user.clone());

        let postgresql_auth = PostgreSQL::new(
            username.to_owned(),
            password.to_owned(),
            password_config.clone(),
            cache_manager.clone(),
        );

        if postgresql_auth.apply().await? {
            return Ok(true);
        }
    }

    Ok(false)
}

#[async_trait]
impl Authentication for PostgreSQL {
    async fn apply(&self) -> Result<bool, MqttBrokerError> {
        if let Some(user) = self.cache_manager.user_info.get(&self.username) {
            // get stored hash and salt from user data
            let stored_hash = &user.password;
            let salt = user.salt.as_deref().unwrap_or("");

            return self.verify_password(stored_hash, &self.password, salt);
        }
        Err(MqttBrokerError::UserDoesNotExist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::tool::test_build_mqtt_cache_manager;
    use common_base::tools::now_second;
    use common_config::security::PasswordConfig;
    use metadata_struct::mqtt::user::MqttUser;

    #[tokio::test]
    async fn test_postgresql_plain_authentication() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let username = "test_user".to_string();
        let password = "test_password".to_string();

        // create user
        let user = MqttUser {
            username: username.clone(),
            password: password.clone(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };
        cache_manager.add_user(user);

        // configure to plaintext verification
        let password_config = PasswordConfig {
            credential_type: "password".to_string(),
            algorithm: "plain".to_string(),
            salt_position: Some("disable".to_string()),
            salt_rounds: None,
            mac_fun: None,
            iterations: None,
            dk_length: None,
        };

        let postgresql_auth = PostgreSQL::new(
            username.clone(),
            password.clone(),
            password_config,
            cache_manager.clone(),
        );

        let result = postgresql_auth.apply().await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_postgresql_md5_authentication() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let username = "test_user".to_string();
        let plain_password = "test_password";

        // pre-calculate MD5 hash
        let stored_hash = format!("{:x}", md5::compute(plain_password.as_bytes()));

        // create user (stored hash password)
        let user = MqttUser {
            username: username.clone(),
            password: stored_hash,
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };
        cache_manager.add_user(user);

        // configure to MD5 verification
        let password_config = PasswordConfig {
            credential_type: "password".to_string(),
            algorithm: "md5".to_string(),
            salt_position: Some("disable".to_string()),
            salt_rounds: None,
            mac_fun: None,
            iterations: None,
            dk_length: None,
        };

        let postgresql_auth = PostgreSQL::new(
            username.clone(),
            plain_password.to_string(),
            password_config,
            cache_manager.clone(),
        );

        let result = postgresql_auth.apply().await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_postgresql_bcrypt_authentication() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let username = "test_user".to_string();
        let plain_password = "test_password";

        // pre-calculate bcrypt hash
        let stored_hash = bcrypt::hash(plain_password, bcrypt::DEFAULT_COST).unwrap();

        // create user (stored hash password)
        let user = MqttUser {
            username: username.clone(),
            password: stored_hash,
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };
        cache_manager.add_user(user);

        // configure to bcrypt verification
        let password_config = PasswordConfig {
            credential_type: "password".to_string(),
            algorithm: "bcrypt".to_string(),
            salt_position: None,
            salt_rounds: Some(12),
            mac_fun: None,
            iterations: None,
            dk_length: None,
        };

        let postgresql_auth = PostgreSQL::new(
            username.clone(),
            plain_password.to_string(),
            password_config,
            cache_manager.clone(),
        );

        let result = postgresql_auth.apply().await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_postgresql_sha256_with_salt() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let username = "test_user".to_string();
        let plain_password = "test_password";
        let salt = "random_salt";

        // pre-calculate SHA256 hash with salt suffix
        let password_with_salt = format!("{}{}", plain_password, salt);
        let mut hasher = Sha256::new();
        hasher.update(password_with_salt.as_bytes());
        let stored_hash = format!("{:x}", hasher.finalize());

        // create user (stored hash password with salt)
        let user = MqttUser {
            username: username.clone(),
            password: stored_hash,
            salt: Some(salt.to_string()),
            is_superuser: false,
            create_time: now_second(),
        };
        cache_manager.add_user(user);

        // configure to SHA256 verification with salt suffix
        let password_config = PasswordConfig {
            credential_type: "password".to_string(),
            algorithm: "sha256".to_string(),
            salt_position: Some("suffix".to_string()),
            salt_rounds: None,
            mac_fun: None,
            iterations: None,
            dk_length: None,
        };

        let postgresql_auth = PostgreSQL::new(
            username.clone(),
            plain_password.to_string(),
            password_config,
            cache_manager.clone(),
        );

        let result = postgresql_auth.apply().await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    #[allow(unused_must_use)]
    async fn test_postgresql_pbkdf2_authentication() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let username = "test_user".to_string();
        let plain_password = "test_password";
        let salt = "test_salt";
        let iterations = 10000u32;
        let dk_length = 32;

        // pre-calculate PBKDF2 hash
        let mut derived_key = vec![0u8; dk_length];
        pbkdf2::<Hmac<Sha256>>(
            plain_password.as_bytes(),
            salt.as_bytes(),
            iterations,
            &mut derived_key,
        );
        let stored_hash = hex::encode(derived_key);

        // create user (stored PBKDF2 hash with salt)
        let user = MqttUser {
            username: username.clone(),
            password: stored_hash,
            salt: Some(salt.to_string()),
            is_superuser: false,
            create_time: now_second(),
        };
        cache_manager.add_user(user);

        // configure to PBKDF2 verification
        let password_config = PasswordConfig {
            credential_type: "password".to_string(),
            algorithm: "pbkdf2".to_string(),
            salt_position: Some("suffix".to_string()),
            salt_rounds: None,
            mac_fun: Some("sha256".to_string()),
            iterations: Some(iterations),
            dk_length: Some(dk_length as u32),
        };

        let postgresql_auth = PostgreSQL::new(
            username.clone(),
            plain_password.to_string(),
            password_config,
            cache_manager.clone(),
        );

        let result = postgresql_auth.apply().await.unwrap();
        assert!(result);
    }
}
