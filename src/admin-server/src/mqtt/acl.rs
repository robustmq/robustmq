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

use crate::{
    extractor::ValidatedJson,
    state::HttpState,
    tool::{
        query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct AclListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateAclReq {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource type length must be between 1-50"
    ))]
    #[validate(custom(function = "validate_acl_resource_type"))]
    pub resource_type: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Resource name length must be between 1-256"
    ))]
    pub resource_name: String,

    #[validate(length(min = 1, max = 256, message = "Topic length must be between 1-256"))]
    pub topic: String,

    #[validate(length(max = 128, message = "IP length cannot exceed 128"))]
    pub ip: String,

    #[validate(length(min = 1, max = 50, message = "Action length must be between 1-50"))]
    #[validate(custom(function = "validate_acl_action"))]
    pub action: String,

    #[validate(length(min = 1, max = 50, message = "Permission length must be between 1-50"))]
    #[validate(custom(function = "validate_acl_permission"))]
    pub permission: String,
}

fn validate_acl_resource_type(resource_type: &str) -> Result<(), validator::ValidationError> {
    match resource_type {
        "ClientId" | "Username" | "IpAddress" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_acl_resource_type");
            err.message = Some(std::borrow::Cow::from(
                "Resource type must be ClientId, Username or IpAddress",
            ));
            Err(err)
        }
    }
}

fn validate_acl_action(action: &str) -> Result<(), validator::ValidationError> {
    match action {
        "Publish" | "Subscribe" | "All" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_acl_action");
            err.message = Some(std::borrow::Cow::from(
                "Action must be Publish, Subscribe or All",
            ));
            Err(err)
        }
    }
}

fn validate_acl_permission(permission: &str) -> Result<(), validator::ValidationError> {
    match permission {
        "Allow" | "Deny" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_acl_permission");
            err.message = Some(std::borrow::Cow::from("Permission must be Allow or Deny"));
            Err(err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteAclReq {
    #[validate(length(min = 1, max = 50, message = "Resource type length must be between 1-50"))]
    #[validate(custom(function = "validate_acl_resource_type"))]
    pub resource_type: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Resource name length must be between 1-256"
    ))]
    pub resource_name: String,

    #[validate(length(min = 1, max = 256, message = "Topic length must be between 1-256"))]
    pub topic: String,

    #[validate(length(max = 128, message = "IP length cannot exceed 128"))]
    pub ip: String,

    #[validate(length(min = 1, max = 50, message = "Action length must be between 1-50"))]
    #[validate(custom(function = "validate_acl_action"))]
    pub action: String,

    #[validate(length(min = 1, max = 50, message = "Permission length must be between 1-50"))]
    #[validate(custom(function = "validate_acl_permission"))]
    pub permission: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AclListRow {
    pub resource_type: String,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: String,
    pub permission: String,
}

use common_base::{
    enum_type::mqtt::acl::{
        mqtt_acl_action::MqttAclAction, mqtt_acl_permission::MqttAclPermission,
        mqtt_acl_resource_type::MqttAclResourceType,
    },
    error::{common::CommonError, ResultCommonError},
    http_response::{error_response, success_response},
};
use metadata_struct::acl::mqtt_acl::MqttAcl;
use mqtt_broker::security::AuthDriver;
use std::{str::FromStr, sync::Arc};

pub async fn acl_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<AclListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );
    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );
    let data = match auth_driver.read_all_acl().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let mut acls_list = Vec::new();
    for acl in data {
        acls_list.push(AclListRow {
            resource_type: acl.resource_type.to_string(),
            resource_name: acl.resource_name.to_string(),
            topic: acl.topic.to_string(),
            ip: acl.ip.to_string(),
            action: acl.action.to_string(),
            permission: acl.permission.to_string(),
        });
    }

    let filtered = apply_filters(acls_list, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for AclListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "resource_type" => Some(self.resource_type.clone()),
            "resource_name" => Some(self.resource_type.clone()),
            "topic" => Some(self.resource_type.clone()),
            "ip" => Some(self.resource_type.clone()),
            _ => None,
        }
    }
}

pub async fn acl_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateAclReq>,
) -> String {
    match acl_create_inner(&state, &params).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

async fn acl_create_inner(state: &Arc<HttpState>, params: &CreateAclReq) -> ResultCommonError {
    let resource_type = match MqttAclResourceType::from_str(&params.resource_type) {
        Ok(data) => data,
        Err(e) => {
            return Err(CommonError::CommonError(e));
        }
    };

    let action = match MqttAclAction::from_str(&params.action) {
        Ok(data) => data,
        Err(e) => {
            return Err(CommonError::CommonError(e));
        }
    };

    let permission = match MqttAclPermission::from_str(&params.permission) {
        Ok(data) => data,
        Err(e) => {
            return Err(CommonError::CommonError(e));
        }
    };

    let mqtt_acl = MqttAcl {
        resource_type,
        resource_name: params.resource_name.clone(),
        topic: params.topic.clone(),
        ip: params.ip.clone(),
        action,
        permission,
    };
    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );
    if let Err(e) = auth_driver.save_acl(mqtt_acl).await {
        return Err(CommonError::CommonError(e.to_string()));
    }
    Ok(())
}

pub async fn acl_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteAclReq>,
) -> String {
    match acl_delete_inner(&state, &params).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

async fn acl_delete_inner(state: &Arc<HttpState>, params: &DeleteAclReq) -> ResultCommonError {
    let resource_type = match MqttAclResourceType::from_str(&params.resource_type) {
        Ok(data) => data,
        Err(e) => {
            return Err(CommonError::CommonError(e));
        }
    };

    let action = match MqttAclAction::from_str(&params.action) {
        Ok(data) => data,
        Err(e) => {
            return Err(CommonError::CommonError(e));
        }
    };

    let permission = match MqttAclPermission::from_str(&params.permission) {
        Ok(data) => data,
        Err(e) => {
            return Err(CommonError::CommonError(e));
        }
    };

    let mqtt_acl = MqttAcl {
        resource_type,
        resource_name: params.resource_name.clone(),
        topic: params.topic.clone(),
        ip: params.ip.clone(),
        action,
        permission,
    };
    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );
    if let Err(e) = auth_driver.delete_acl(mqtt_acl).await {
        return Err(CommonError::CommonError(e.to_string()));
    }
    Ok(())
}
