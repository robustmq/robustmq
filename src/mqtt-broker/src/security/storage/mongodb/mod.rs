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

use std::str::FromStr;

use crate::core::error::MqttBrokerError;
use crate::security::AuthStorageAdapter;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::get_blacklist_type_by_str;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::tools::now_second;
use dashmap::DashMap;
use futures::TryStreamExt;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auth::storage::MongoDBConfig;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::tenant::DEFAULT_TENANT;
use mongodb::bson::{Bson, Document};
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection};
use tracing::warn;

pub struct MongoDBAuthStorageAdapter {
    config: MongoDBConfig,
}

impl MongoDBAuthStorageAdapter {
    pub fn new(config: MongoDBConfig) -> Self {
        Self { config }
    }

    async fn collection(&self, name: &str) -> Result<Collection<Document>, MqttBrokerError> {
        let mut options = ClientOptions::parse(&self.config.mongodb_uri)
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?;

        if !self.config.username.trim().is_empty() {
            options.credential = Some(
                mongodb::options::Credential::builder()
                    .username(Some(self.config.username.clone()))
                    .password(Some(self.config.password.clone()))
                    .build(),
            );
        }

        let client = Client::with_options(options)
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?;
        Ok(client.database(&self.config.database).collection(name))
    }

    fn parse_filter(query: &str) -> Result<Document, MqttBrokerError> {
        if query.trim().is_empty() {
            return Ok(Document::new());
        }
        serde_json::from_str::<Document>(query)
            .map_err(|e| MqttBrokerError::BsonSerializationError(e.to_string()))
    }

    fn parse_created_to_seconds(v: Option<&Bson>) -> u64 {
        match v {
            Some(Bson::DateTime(dt)) => dt.timestamp_millis().max(0) as u64 / 1000,
            Some(Bson::Int64(ts)) if *ts >= 0 => *ts as u64,
            Some(Bson::Int32(ts)) if *ts >= 0 => *ts as u64,
            Some(Bson::String(s)) => {
                if let Ok(ts) = s.parse::<u64>() {
                    return ts;
                }
                if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                    return dt.and_utc().timestamp().max(0) as u64;
                }
                now_second()
            }
            _ => now_second(),
        }
    }

    fn parse_is_superuser(v: Option<&Bson>) -> bool {
        match v {
            Some(Bson::Boolean(v)) => *v,
            Some(Bson::Int32(v)) => *v == 1,
            Some(Bson::Int64(v)) => *v == 1,
            Some(Bson::String(v)) => v == "1" || v.eq_ignore_ascii_case("true"),
            _ => false,
        }
    }

    fn parse_permission(v: Option<&Bson>) -> Result<MqttAclPermission, MqttBrokerError> {
        match v {
            Some(Bson::Int32(0)) | Some(Bson::Int64(0)) => Ok(MqttAclPermission::Deny),
            Some(Bson::Int32(1)) | Some(Bson::Int64(1)) => Ok(MqttAclPermission::Allow),
            Some(Bson::String(s)) => {
                MqttAclPermission::from_str(s).map_err(|_| MqttBrokerError::InvalidAclPermission)
            }
            _ => Err(MqttBrokerError::InvalidAclPermission),
        }
    }

    fn parse_action(v: Option<&Bson>) -> Result<MqttAclAction, MqttBrokerError> {
        match v {
            Some(Bson::Int32(0)) | Some(Bson::Int64(0)) => Ok(MqttAclAction::All),
            Some(Bson::Int32(1)) | Some(Bson::Int64(1)) => Ok(MqttAclAction::Subscribe),
            Some(Bson::Int32(2)) | Some(Bson::Int64(2)) => Ok(MqttAclAction::Publish),
            Some(Bson::Int32(3)) | Some(Bson::Int64(3)) => Ok(MqttAclAction::PubSub),
            Some(Bson::Int32(4)) | Some(Bson::Int64(4)) => Ok(MqttAclAction::Retain),
            Some(Bson::Int32(5)) | Some(Bson::Int64(5)) => Ok(MqttAclAction::Qos),
            Some(Bson::String(s)) => {
                let normalized = match s.to_ascii_lowercase().as_str() {
                    "all" => "All",
                    "subscribe" => "Subscribe",
                    "publish" => "Publish",
                    "pubsub" | "publish_subscribe" => "PubSub",
                    "retain" => "Retain",
                    "qos" => "Qos",
                    _ => s,
                };
                MqttAclAction::from_str(normalized).map_err(|_| MqttBrokerError::InvalidAclAction)
            }
            _ => Err(MqttBrokerError::InvalidAclAction),
        }
    }

    fn extract_topics(doc: &Document) -> Vec<String> {
        if let Some(Bson::String(topic)) = doc.get("topic") {
            return vec![topic.clone()];
        }
        if let Some(Bson::Array(topics)) = doc.get("topics") {
            return topics
                .iter()
                .filter_map(|v| match v {
                    Bson::String(topic) => Some(topic.clone()),
                    _ => None,
                })
                .collect();
        }
        Vec::new()
    }

    fn parse_blacklist_type(v: Option<&Bson>) -> Result<String, MqttBrokerError> {
        match v {
            Some(Bson::String(s)) => {
                let normalized = match s.to_ascii_lowercase().as_str() {
                    "clientid" | "client_id" => "ClientId",
                    "user" | "username" => "User",
                    "ip" | "ipaddr" | "ipaddress" => "Ip",
                    "clientidmatch" | "client_id_match" => "ClientIdMatch",
                    "usermatch" | "user_match" => "UserMatch",
                    "ipcidr" | "ip_cidr" => "IPCIDR",
                    _ => s,
                };
                Ok(normalized.to_string())
            }
            _ => Err(MqttBrokerError::CommonError(
                "missing or invalid blacklist_type".to_string(),
            )),
        }
    }
}

#[async_trait]
impl AuthStorageAdapter for MongoDBAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let collection = self.collection(&self.config.collection_user).await?;
        let filter = Self::parse_filter(&self.config.query_user)?;
        let mut cursor = collection
            .find(filter, None)
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?;

        let results = DashMap::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?
        {
            let Some(Bson::String(username)) = doc.get("username") else {
                warn!("MongoDB user record missing username, skip");
                continue;
            };
            let Some(Bson::String(password)) = doc.get("password") else {
                warn!(username = %username, "MongoDB user record missing password, skip");
                continue;
            };

            let user = MqttUser {
                tenant: DEFAULT_TENANT.to_string(),
                username: username.clone(),
                password: password.clone(),
                salt: doc.get_str("salt").ok().map(|v| v.to_string()),
                is_superuser: Self::parse_is_superuser(doc.get("is_superuser")),
                create_time: Self::parse_created_to_seconds(doc.get("created")),
            };
            results.insert(username.clone(), user);
        }

        Ok(results)
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let collection = self.collection(&self.config.collection_acl).await?;
        let filter = Self::parse_filter(&self.config.query_acl)?;
        let mut cursor = collection
            .find(filter, None)
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?;

        let mut results = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?
        {
            let username = doc.get_str("username").unwrap_or_default().to_string();
            let clientid = doc.get_str("clientid").unwrap_or_default().to_string();
            let ip = doc
                .get_str("ipaddr")
                .or_else(|_| doc.get_str("ipaddress"))
                .unwrap_or_default()
                .to_string();
            let permission = match Self::parse_permission(doc.get("permission")) {
                Ok(v) => v,
                Err(_) => {
                    warn!("MongoDB ACL record has invalid permission, skip");
                    continue;
                }
            };
            let action = match Self::parse_action(doc.get("access").or_else(|| doc.get("action"))) {
                Ok(v) => v,
                Err(_) => {
                    warn!("MongoDB ACL record has invalid action/access, skip");
                    continue;
                }
            };
            let topics = Self::extract_topics(&doc);
            if topics.is_empty() {
                warn!("MongoDB ACL record missing topic/topics, skip");
                continue;
            }

            let resource_type = if username.is_empty() {
                MqttAclResourceType::ClientId
            } else {
                MqttAclResourceType::User
            };
            let resource_name = if username.is_empty() {
                clientid.clone()
            } else {
                username.clone()
            };

            for topic in topics {
                results.push(MqttAcl {
                    tenant: DEFAULT_TENANT.to_string(),
                    resource_type,
                    resource_name: resource_name.clone(),
                    topic,
                    ip: ip.clone(),
                    action,
                    permission,
                });
            }
        }
        Ok(results)
    }

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        let collection = self.collection(&self.config.collection_blacklist).await?;
        let filter = Self::parse_filter(&self.config.query_blacklist)?;
        let mut cursor = collection
            .find(filter, None)
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?;

        let mut results = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?
        {
            let blacklist_type = match Self::parse_blacklist_type(doc.get("blacklist_type")) {
                Ok(v) => v,
                Err(_) => {
                    warn!("MongoDB blacklist record has invalid blacklist_type, skip");
                    continue;
                }
            };
            let Some(Bson::String(resource_name)) = doc.get("resource_name") else {
                warn!("MongoDB blacklist record missing resource_name, skip");
                continue;
            };

            let end_time = match doc.get("end_time") {
                Some(Bson::Int64(v)) if *v >= 0 => *v as u64,
                Some(Bson::Int32(v)) if *v >= 0 => *v as u64,
                Some(Bson::String(v)) => v.parse::<u64>().unwrap_or(0),
                _ => 0,
            };
            let desc = doc.get_str("desc").unwrap_or_default().to_string();

            let blacklist = MqttAclBlackList {
                tenant: DEFAULT_TENANT.to_string(),
                blacklist_type: get_blacklist_type_by_str(&blacklist_type)?,
                resource_name: resource_name.clone(),
                end_time,
                desc,
            };
            results.push(blacklist);
        }
        Ok(results)
    }
}
