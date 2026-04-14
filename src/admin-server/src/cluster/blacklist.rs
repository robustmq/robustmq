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
    state::HttpState,
    tool::extractor::ValidatedJson,
    tool::{
        query::{apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlackListListReq {
    pub tenant: Option<String>,
    pub name: Option<String>,
    pub resource_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateBlackListReq {
    #[validate(length(min = 1, max = 128, message = "Name length must be between 1-128"))]
    pub name: String,

    #[validate(length(min = 1, max = 64, message = "Tenant length must be between 1-64"))]
    pub tenant: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Blacklist type length must be between 1-50"
    ))]
    #[validate(custom(function = "validate_blacklist_type"))]
    pub blacklist_type: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Resource name length must be between 1-256"
    ))]
    pub resource_name: String,

    #[validate(range(min = 1, message = "End time must be greater than 0"))]
    pub end_time: u64,

    #[validate(length(max = 500, message = "Description length cannot exceed 500 characters"))]
    pub desc: Option<String>,
}

fn validate_blacklist_type(blacklist_type: &str) -> Result<(), validator::ValidationError> {
    match blacklist_type {
        "ClientId" | "User" | "Ip" | "ClientIdMatch" | "UserMatch" | "IPCIDR" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_blacklist_type");
            err.message = Some(std::borrow::Cow::from(
                "Blacklist type must be ClientId, User, Ip, ClientIdMatch, UserMatch or IPCIDR",
            ));
            Err(err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteBlackListReq {
    #[validate(length(min = 1, max = 64, message = "Tenant length must be between 1-64"))]
    pub tenant: String,

    #[validate(length(min = 1, max = 128, message = "Name length must be between 1-128"))]
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlackListListRow {
    pub name: String,
    pub tenant: String,
    pub blacklist_type: String,
    pub resource_name: String,
    pub end_time: String,
    pub desc: String,
}

use common_base::{
    http_response::{error_response, success_response},
    utils::time_util::timestamp_to_local_datetime,
};
use common_security::storage::blacklist::BlackListStorage;
use metadata_struct::auth::blacklist::{get_blacklist_type_by_str, SecurityBlackList};
use std::sync::Arc;

pub async fn blacklist_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<BlackListListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        None,
        None,
        None,
    );

    let data: Vec<SecurityBlackList> = state
        .mqtt_context
        .security_manager
        .metadata
        .get_all_blacklist();

    let blacklists: Vec<BlackListListRow> = data
        .into_iter()
        .filter(|b| {
            if let Some(ref tenant) = params.tenant {
                if !b.tenant.contains(tenant.as_str()) {
                    return false;
                }
            }
            if let Some(ref name) = params.name {
                if !b.name.contains(name.as_str()) {
                    return false;
                }
            }
            if let Some(ref resource_name) = params.resource_name {
                if !b.resource_name.contains(resource_name.as_str()) {
                    return false;
                }
            }
            true
        })
        .map(|b| BlackListListRow {
            name: b.name.clone(),
            tenant: b.tenant.clone(),
            blacklist_type: b.blacklist_type.to_string(),
            resource_name: b.resource_name.clone(),
            end_time: timestamp_to_local_datetime(b.end_time as i64),
            desc: b.desc.clone(),
        })
        .collect();

    let sorted = apply_sorting(blacklists, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for BlackListListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "tenant" => Some(self.tenant.clone()),
            "blacklist_type" => Some(self.blacklist_type.clone()),
            "resource_name" => Some(self.resource_name.clone()),
            _ => None,
        }
    }
}

pub async fn blacklist_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateBlackListReq>,
) -> String {
    let blacklist_type = match get_blacklist_type_by_str(&params.blacklist_type) {
        Ok(blacklist_type) => blacklist_type,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let mqtt_blacklist = SecurityBlackList {
        name: params.name.clone(),
        tenant: params.tenant.clone(),
        blacklist_type,
        resource_name: params.resource_name.clone(),
        end_time: params.end_time,
        desc: params.desc.clone().unwrap_or_default(),
    };

    let blacklist_storage = BlackListStorage::new(state.client_pool.clone());
    match blacklist_storage.save_blacklist(mqtt_blacklist).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn blacklist_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteBlackListReq>,
) -> String {
    let blacklist_storage = BlackListStorage::new(state.client_pool.clone());
    match blacklist_storage
        .delete_blacklist(&params.tenant, &params.name)
        .await
    {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}
