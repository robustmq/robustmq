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
use axum::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use common_config::security::JwtConfig;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use metadata_struct::mqtt::user::MqttUser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct JwtAuth {
    username: String,
    password: String,
    jwt_config: JwtConfig,
    cache_manager: Arc<MQTTCacheManager>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: Option<String>,        // Subject (user ID)
    pub username: Option<String>,   // Username
    pub exp: Option<usize>,         // Expiration time
    pub iat: Option<usize>,         // Issued at
    pub is_superuser: Option<bool>, // Is superuser
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>, // Other claims
}

impl JwtAuth {
    pub fn new(
        username: String,
        password: String,
        jwt_config: JwtConfig,
        cache_manager: Arc<MQTTCacheManager>,
    ) -> Self {
        JwtAuth {
            username,
            password,
            jwt_config,
            cache_manager,
        }
    }

    /// get JWT token from config
    fn get_jwt_token(&self) -> &str {
        match self.jwt_config.jwt_source.as_str() {
            "username" => &self.username,
            "password" => &self.password,
            _ => &self.password, // default from password
        }
    }

    /// create decoding key based on encryption method
    fn create_decoding_key(&self) -> Result<DecodingKey, MqttBrokerError> {
        match self.jwt_config.jwt_encryption.as_str() {
            "hmac-based" => {
                let secret = self
                    .jwt_config
                    .secret
                    .as_ref()
                    .ok_or(MqttBrokerError::JwtSecretNotFound)?;

                let secret_bytes = if self.jwt_config.secret_base64_encoded.unwrap_or(false) {
                    BASE64_STANDARD
                        .decode(secret)
                        .map_err(|e| MqttBrokerError::JwtSecretDecodeError(e.to_string()))?
                } else {
                    secret.as_bytes().to_vec()
                };

                Ok(DecodingKey::from_secret(&secret_bytes))
            }
            "public-key" => {
                let public_key = self
                    .jwt_config
                    .public_key
                    .as_ref()
                    .ok_or(MqttBrokerError::JwtPublicKeyNotFound)?;

                DecodingKey::from_rsa_pem(public_key.as_bytes())
                    .or_else(|_| DecodingKey::from_ec_pem(public_key.as_bytes()))
                    .map_err(|e| MqttBrokerError::JwtPublicKeyDecodeError(e.to_string()))
            }
            _ => Err(MqttBrokerError::UnsupportedJwtEncryption(
                self.jwt_config.jwt_encryption.clone(),
            )),
        }
    }

    /// get validation algorithm based on encryption method
    fn get_validation_algorithm(&self) -> Result<Algorithm, MqttBrokerError> {
        match self.jwt_config.jwt_encryption.as_str() {
            "hmac-based" => Ok(Algorithm::HS256), // default use HS256
            "public-key" => Ok(Algorithm::RS256), // default use RS256
            _ => Err(MqttBrokerError::UnsupportedJwtEncryption(
                self.jwt_config.jwt_encryption.clone(),
            )),
        }
    }

    /// verify JWT token
    fn verify_jwt(&self, token: &str) -> Result<JwtClaims, MqttBrokerError> {
        let decoding_key = self.create_decoding_key()?;
        let algorithm = self.get_validation_algorithm()?;

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true; // verify expiration time

        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)
            .map_err(|e| MqttBrokerError::JwtVerificationError(e.to_string()))?;

        Ok(token_data.claims)
    }
}

/// JWT authentication check entry function
pub async fn jwt_check_login(
    cache_manager: &Arc<MQTTCacheManager>,
    jwt_config: &JwtConfig,
    username: &str,
    password: &str,
) -> Result<bool, MqttBrokerError> {
    let jwt_auth = JwtAuth::new(
        username.to_owned(),
        password.to_owned(),
        jwt_config.clone(),
        cache_manager.clone(),
    );

    // Pure JWT validation without storage fallback
    match jwt_auth.apply().await {
        Ok(flag) => Ok(flag),
        Err(e) => Err(e),
    }
}

#[async_trait]
impl Authentication for JwtAuth {
    async fn apply(&self) -> Result<bool, MqttBrokerError> {
        let jwt_token = self.get_jwt_token();

        // verify JWT token
        let claims = self.verify_jwt(jwt_token)?;

        // get username from JWT claims
        let jwt_username = claims
            .username
            .or(claims.sub)
            .unwrap_or_else(|| self.username.clone());

        // update user information to cache
        let user = MqttUser {
            username: jwt_username.clone(),
            password: self.password.clone(),
            salt: None,
            is_superuser: claims.is_superuser.unwrap_or(false),
        };
        self.cache_manager.add_user(user);

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[tokio::test]
    async fn test_jwt_hmac_authentication() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let secret = "test_secret";

        // create JWT config
        let jwt_config = JwtConfig {
            jwt_source: "password".to_string(),
            jwt_encryption: "hmac-based".to_string(),
            secret: Some(secret.to_string()),
            secret_base64_encoded: Some(false),
            public_key: None,
        };

        // create test JWT claims
        let claims = JwtClaims {
            sub: Some("test_user".to_string()),
            username: Some("test_user".to_string()),
            exp: Some((chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize),
            iat: Some(chrono::Utc::now().timestamp() as usize),
            is_superuser: Some(false),
            other: serde_json::Map::new(),
        };

        // generate JWT token
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let token = encode(&header, &claims, &encoding_key).unwrap();

        // create JWT authenticator
        let jwt_auth = JwtAuth::new(
            "test_user".to_string(),
            token,
            jwt_config,
            cache_manager.clone(),
        );

        // test authentication
        let result = jwt_auth.apply().await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // verify user is added to cache
        assert!(cache_manager.user_info.contains_key("test_user"));
    }

    #[tokio::test]
    async fn test_jwt_from_username() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let secret = "test_secret";

        // create JWT config, get token from username
        let jwt_config = JwtConfig {
            jwt_source: "username".to_string(),
            jwt_encryption: "hmac-based".to_string(),
            secret: Some(secret.to_string()),
            secret_base64_encoded: Some(false),
            public_key: None,
        };

        // create test JWT claims
        let claims = JwtClaims {
            sub: Some("test_user".to_string()),
            username: Some("test_user".to_string()),
            exp: Some((chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize),
            iat: Some(chrono::Utc::now().timestamp() as usize),
            is_superuser: Some(true),
            other: serde_json::Map::new(),
        };

        // generate JWT token
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let token = encode(&header, &claims, &encoding_key).unwrap();

        // create JWT authenticator, token in username field
        let jwt_auth = JwtAuth::new(
            token, // JWT token in username field
            "test_password".to_string(),
            jwt_config,
            cache_manager.clone(),
        );

        // test authentication
        let result = jwt_auth.apply().await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // verify superuser permission
        let user = cache_manager.user_info.get("test_user").unwrap();
        assert!(user.is_superuser);
    }

    #[test]
    fn test_jwt_claims_deserialization() {
        let json_claims = r#"
        {
            "sub": "user123",
            "username": "test_user",
            "exp": 1234567890,
            "iat": 1234567800,
            "is_superuser": true,
            "custom_field": "custom_value"
        }
        "#;

        let claims: JwtClaims = serde_json::from_str(json_claims).unwrap();
        assert_eq!(claims.sub, Some("user123".to_string()));
        assert_eq!(claims.username, Some("test_user".to_string()));
        assert_eq!(claims.exp, Some(1234567890));
        assert_eq!(claims.iat, Some(1234567800));
        assert_eq!(claims.is_superuser, Some(true));
        assert_eq!(claims.other.get("custom_field").unwrap(), "custom_value");
    }

    #[tokio::test]
    async fn test_jwt_invalid_token() {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let jwt_config = JwtConfig {
            jwt_source: "password".to_string(),
            jwt_encryption: "hmac-based".to_string(),
            secret: Some("test_secret".to_string()),
            secret_base64_encoded: Some(false),
            public_key: None,
        };

        let jwt_auth = JwtAuth::new(
            "test_user".to_string(),
            "invalid_token".to_string(), // invalid JWT token
            jwt_config,
            cache_manager,
        );

        let result = jwt_auth.apply().await;
        assert!(result.is_err());
    }
}
