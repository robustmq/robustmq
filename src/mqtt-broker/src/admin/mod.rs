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

use crate::handler::cache::CacheManager;
use crate::handler::flapping_detect::enable_flapping_detect;
use crate::observability::slow::sub::{enable_slow_sub, read_slow_sub_record, SlowSubData};
use crate::security::AuthDriver;
use crate::server::connection_manager::ConnectionManager;
use crate::storage::topic::TopicStorage;
use crate::{handler::error::MqttBrokerError, storage::cluster::ClusterStorage};
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::tools::serialize_value;
use common_base::utils::file_utils::get_project_root;
use common_base::utils::time_util::get_current_millisecond_timestamp;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, CreateAclReply, CreateAclRequest, CreateBlacklistReply,
    CreateBlacklistRequest, CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest,
    CreateUserReply, CreateUserRequest, DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply,
    DeleteBlacklistRequest, DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest,
    DeleteUserReply, DeleteUserRequest, EnableFlappingDetectReply, EnableFlappingDetectRequest,
    EnableSlowSubScribeReply, EnableSlowSubscribeRequest, ListAclReply, ListBlacklistReply,
    ListConnectionRaw, ListConnectionReply, ListSlowSubScribeRaw, ListSlowSubscribeReply,
    ListSlowSubscribeRequest, ListTopicReply, ListTopicRequest, ListUserReply, MqttTopic,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn cluster_status_by_req(
    client_pool: &Arc<ClientPool>,
) -> Result<ClusterStatusReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

    let mut broker_node_list = Vec::new();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage.node_list().await?;
    for node in data {
        broker_node_list.push(format!("{}@{}", node.node_ip, node.node_id));
    }
    Ok(ClusterStatusReply {
        nodes: broker_node_list,
        cluster_name: config.cluster_name.clone(),
    })
}

pub async fn create_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateUserRequest>,
) -> Result<Response<CreateUserReply>, Status> {
    let req = request.into_inner();
    let mqtt_user = MqttUser {
        username: req.username,
        password: req.password,
        is_superuser: req.is_superuser,
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.save_user(mqtt_user).await {
        Ok(_) => Ok(Response::new(CreateUserReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn delete_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteUserRequest>,
) -> Result<Response<DeleteUserReply>, Status> {
    let req = request.into_inner();

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.delete_user(req.username).await {
        Ok(_) => Ok(Response::new(DeleteUserReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn list_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<Response<ListUserReply>, Status> {
    let mut reply = ListUserReply::default();
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.read_all_user().await {
        Ok(data) => {
            let mut users = Vec::new();
            for ele in data {
                users.push(ele.1.encode());
            }
            reply.users = users;
            Ok(Response::new(reply))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn list_acl_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<Response<ListAclReply>, Status> {
    let mut reply = ListAclReply::default();

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.read_all_acl().await {
        Ok(data) => {
            let mut acls_list = Vec::new();
            // todo finish get_items
            for ele in data {
                match ele.encode() {
                    Ok(acl) => acls_list.push(acl),
                    Err(e) => return Err(Status::cancelled(e.to_string())),
                }
            }
            reply.acls = acls_list;
            Ok(Response::new(reply))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn create_acl_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateAclRequest>,
) -> Result<Response<CreateAclReply>, Status> {
    let req = request.into_inner();

    let mqtt_acl = match MqttAcl::decode(&req.acl) {
        Ok(acl) => acl,
        Err(e) => return Err(Status::cancelled(e.to_string())),
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.save_acl(mqtt_acl).await {
        Ok(_) => Ok(Response::new(CreateAclReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn delete_acl_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteAclRequest>,
) -> Result<Response<DeleteAclReply>, Status> {
    let req = request.into_inner();
    let mqtt_acl = match MqttAcl::decode(&req.acl) {
        Ok(acl) => acl,
        Err(e) => return Err(Status::cancelled(e.to_string())),
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.delete_acl(mqtt_acl).await {
        Ok(_) => Ok(Response::new(DeleteAclReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn list_blacklist_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<Response<ListBlacklistReply>, Status> {
    let mut reply = ListBlacklistReply::default();
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.read_all_blacklist().await {
        Ok(data) => {
            let mut blacklists = Vec::new();
            for ele in data {
                match ele.encode() {
                    Ok(blacklist) => blacklists.push(blacklist),
                    Err(e) => return Err(Status::cancelled(e.to_string())),
                }
            }
            reply.blacklists = blacklists;
            Ok(Response::new(reply))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn delete_blacklist_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteBlacklistRequest>,
) -> Result<Response<DeleteBlacklistReply>, Status> {
    let req = request.into_inner();
    let mqtt_blacklist = MqttAclBlackList {
        blacklist_type: match req.blacklist_type.as_str() {
            "ClientId" => MqttAclBlackListType::ClientId,
            "User" => MqttAclBlackListType::User,
            "Ip" => MqttAclBlackListType::Ip,
            "ClientIdMatch" => MqttAclBlackListType::ClientIdMatch,
            "UserMatch" => MqttAclBlackListType::UserMatch,
            "IPCIDR" => MqttAclBlackListType::IPCIDR,
            _ => return Err(Status::cancelled("invalid blacklist type".to_string())),
        },
        resource_name: req.resource_name,
        end_time: 0,
        desc: "".to_string(),
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.delete_blacklist(mqtt_blacklist).await {
        Ok(_) => Ok(Response::new(DeleteBlacklistReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn create_blacklist_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateBlacklistRequest>,
) -> Result<Response<CreateBlacklistReply>, Status> {
    let req = request.into_inner();
    let mqtt_blacklist = match MqttAclBlackList::decode(&req.blacklist) {
        Ok(blacklist) => blacklist,
        Err(e) => return Err(Status::cancelled(e.to_string())),
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.save_blacklist(mqtt_blacklist).await {
        Ok(_) => Ok(Response::new(CreateBlacklistReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn enable_flapping_detect_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<EnableFlappingDetectRequest>,
) -> Result<Response<EnableFlappingDetectReply>, Status> {
    let req = request.into_inner();

    match enable_flapping_detect(cache_manager, req).await {
        Ok(_) => Ok(Response::new(EnableFlappingDetectReply {
            is_enable: req.is_enable,
        })),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub fn list_connection_by_req(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
) -> Result<Response<ListConnectionReply>, Status> {
    let mut reply = ListConnectionReply::default();
    let mut list_connection_raw: Vec<ListConnectionRaw> = Vec::new();
    for (key, value) in connection_manager.list_connect() {
        if let Some(mqtt_value) = cache_manager.connection_info.clone().get(&key) {
            let mqtt_info = serialize_value(mqtt_value.value())?;
            let raw = ListConnectionRaw {
                connection_id: value.connection_id,
                connection_type: value.connection_type.to_string(),
                protocol: match value.protocol {
                    Some(protocol) => protocol.into(),
                    None => "None".to_string(),
                },
                source_addr: value.addr.to_string(),
                info: mqtt_info,
            };
            list_connection_raw.push(raw);
        }
    }
    reply.list_connection_raw = list_connection_raw;
    Ok(Response::new(reply))
}

pub async fn enable_slow_subscribe_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<EnableSlowSubscribeRequest>,
) -> Result<Response<EnableSlowSubScribeReply>, Status> {
    let subscribe_request = request.into_inner();

    match enable_slow_sub(cache_manager, subscribe_request.is_enable).await {
        Ok(_) => Ok(Response::new(EnableSlowSubScribeReply {
            is_enable: subscribe_request.is_enable,
        })),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub fn list_slow_subscribe_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<ListSlowSubscribeRequest>,
) -> Result<Response<ListSlowSubscribeReply>, Status> {
    let list_slow_subscribe_request = request.into_inner();
    let mut list_slow_subscribe_raw: Vec<ListSlowSubScribeRaw> = Vec::new();
    let mqtt_config = broker_mqtt_conf();
    if cache_manager.get_slow_sub_config().enable {
        let path = mqtt_config.log.log_path.clone();
        let path_buf = get_project_root()?.join(path.replace("./", "") + "/slow_sub.log");
        let deque = read_slow_sub_record(list_slow_subscribe_request, path_buf)?;
        for slow_sub_data in deque {
            match serde_json::from_str::<SlowSubData>(slow_sub_data.as_str()) {
                Ok(data) => {
                    let raw = ListSlowSubScribeRaw {
                        client_id: data.client_id,
                        topic: data.topic,
                        time_ms: data.time_ms,
                        node_info: data.node_info,
                        create_time: data.create_time,
                        sub_name: data.sub_name,
                    };
                    list_slow_subscribe_raw.push(raw);
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }
    Ok(Response::new(ListSlowSubscribeReply {
        list_slow_subscribe_raw,
    }))
}

pub fn list_topic_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<ListTopicRequest>,
) -> Result<Response<ListTopicReply>, Status> {
    let req = request.into_inner();
    let topic_query_result: Vec<MqttTopic> = match req.match_option {
        0 => cache_manager
            .get_topic_by_name(&req.topic_name)
            .into_iter()
            .take(10)
            .map(|entry| MqttTopic {
                topic_id: entry.topic_id.clone(),
                topic_name: entry.topic_name.clone(),
                cluster_name: entry.cluster_name.clone(),
                is_contain_retain_message: entry.retain_message.is_some(),
            })
            .collect(),
        option => cache_manager
            .topic_info
            .iter()
            .filter(|entry| match option {
                1 => entry.value().topic_name.starts_with(&req.topic_name),
                2 => entry.value().topic_name.contains(&req.topic_name),
                _ => false,
            })
            .take(10)
            .map(|entry| MqttTopic {
                topic_id: entry.value().topic_id.clone(),
                topic_name: entry.value().topic_name.clone(),
                cluster_name: entry.value().cluster_name.clone(),
                is_contain_retain_message: entry.value().retain_message.is_some(),
            })
            .collect(),
    };

    let reply = ListTopicReply {
        topics: topic_query_result,
    };

    Ok(Response::new(reply))
}

pub async fn delete_topic_rewrite_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<DeleteTopicRewriteRuleRequest>,
) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
    let req = request.into_inner();
    let topic_storage = TopicStorage::new(client_pool.clone());
    match topic_storage
        .delete_topic_rewrite_rule(req.action.clone(), req.source_topic.clone())
        .await
    {
        Ok(_) => {
            let config = broker_mqtt_conf();
            let key = cache_manager.topic_rewrite_rule_key(
                &config.cluster_name,
                &req.action,
                &req.source_topic,
            );
            cache_manager.topic_rewrite_rule.remove(&key);
            Ok(Response::new(DeleteTopicRewriteRuleReply::default()))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn create_topic_rewrite_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<CreateTopicRewriteRuleRequest>,
) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let topic_rewrite_rule = MqttTopicRewriteRule {
        cluster: config.cluster_name.clone(),
        action: req.action.clone(),
        source_topic: req.source_topic.clone(),
        dest_topic: req.dest_topic.clone(),
        regex: req.regex.clone(),
        timestamp: get_current_millisecond_timestamp(),
    };
    let topic_storage = TopicStorage::new(client_pool.clone());
    match topic_storage
        .create_topic_rewrite_rule(topic_rewrite_rule.clone())
        .await
    {
        Ok(_) => {
            let key = cache_manager.topic_rewrite_rule_key(
                &config.cluster_name,
                &req.action,
                &req.source_topic,
            );
            cache_manager
                .topic_rewrite_rule
                .insert(key, topic_rewrite_rule);
            Ok(Response::new(CreateTopicRewriteRuleReply::default()))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}
