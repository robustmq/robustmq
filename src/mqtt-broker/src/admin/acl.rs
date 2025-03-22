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
use crate::security::AuthDriver;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest, DeleteAclReply,
    DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest, ListAclReply,
    ListBlacklistReply,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

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
