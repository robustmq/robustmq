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
use common_base::http_response::{error_response, success_response};
use common_base::tools::now_second;
use metadata_struct::connector::rule::ETLRule;
use metadata_struct::mq9::forward_rule::{
    ForkFailureStrategy, Mq9ForwardMatcher, Mq9ForwardRule, Mq9ForwardTarget,
};
use metadata_struct::mq9::Priority;
use nats_broker::storage::forward_rule::Mq9ForwardRuleStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ForwardRuleListReq {
    pub tenant: Option<String>,
    pub rule_name: Option<String>,
    pub topic_name: Option<String>,
    pub enabled: Option<bool>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ForwardRuleDetailReq {
    pub tenant: String,
    pub rule_name: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct CreateForwardRuleReq {
    #[validate(length(min = 1, message = "tenant must not be empty"))]
    pub tenant: String,
    #[validate(length(min = 1, message = "rule_name must not be empty"))]
    pub rule_name: String,
    #[validate(length(min = 1, message = "topic_name must not be empty"))]
    pub topic_name: String,
    #[serde(default)]
    pub mail_address_prefixes: Vec<String>,
    #[serde(default)]
    pub any_tags: Vec<String>,
    #[serde(default)]
    pub priorities: Vec<String>,
    #[serde(default)]
    pub sender_prefixes: Vec<String>,
    #[serde(default = "default_true")]
    pub keep_headers: bool,
    #[serde(default)]
    pub on_failure: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Optional inline ETL rule JSON applied to the forked payload.
    #[serde(default)]
    pub etl_rule: Option<ETLRule>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UpdateForwardRuleReq {
    #[validate(length(min = 1, message = "tenant must not be empty"))]
    pub tenant: String,
    #[validate(length(min = 1, message = "rule_name must not be empty"))]
    pub rule_name: String,
    #[validate(length(min = 1, message = "topic_name must not be empty"))]
    pub topic_name: String,
    #[serde(default)]
    pub mail_address_prefixes: Vec<String>,
    #[serde(default)]
    pub any_tags: Vec<String>,
    #[serde(default)]
    pub priorities: Vec<String>,
    #[serde(default)]
    pub sender_prefixes: Vec<String>,
    #[serde(default = "default_true")]
    pub keep_headers: bool,
    #[serde(default)]
    pub on_failure: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub etl_rule: Option<ETLRule>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct DeleteForwardRuleReq {
    #[validate(length(min = 1, message = "tenant must not be empty"))]
    pub tenant: String,
    #[validate(length(min = 1, message = "rule_name must not be empty"))]
    pub rule_name: String,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForwardRuleListRow {
    pub tenant: String,
    pub rule_name: String,
    pub topic_name: String,
    pub mail_address_prefixes: Vec<String>,
    pub any_tags: Vec<String>,
    pub priorities: Vec<String>,
    pub sender_prefixes: Vec<String>,
    pub keep_headers: bool,
    pub on_failure: String,
    pub enabled: bool,
    pub create_time: u64,
    pub update_time: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etl_rule: Option<ETLRule>,
}

impl Queryable for ForwardRuleListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "tenant" => Some(self.tenant.clone()),
            "rule_name" => Some(self.rule_name.clone()),
            "topic_name" => Some(self.topic_name.clone()),
            "on_failure" => Some(self.on_failure.clone()),
            _ => None,
        }
    }
}

fn from_rule(rule: &Mq9ForwardRule) -> ForwardRuleListRow {
    ForwardRuleListRow {
        tenant: rule.tenant.clone(),
        rule_name: rule.rule_name.clone(),
        topic_name: rule.target.topic_name.clone(),
        mail_address_prefixes: rule.matcher.mail_address_prefixes.clone(),
        any_tags: rule.matcher.any_tags.clone(),
        priorities: rule
            .matcher
            .priorities
            .iter()
            .map(|p| p.as_str().to_string())
            .collect(),
        sender_prefixes: rule.matcher.sender_prefixes.clone(),
        keep_headers: rule.target.keep_headers,
        on_failure: failure_strategy_to_str(&rule.target.on_failure).to_string(),
        enabled: rule.enabled,
        create_time: rule.create_time,
        update_time: rule.update_time,
        etl_rule: rule.etl_rule.clone(),
    }
}

fn parse_priority(s: &str) -> Result<Priority, String> {
    Priority::parse(&s.to_lowercase())
        .ok_or_else(|| format!("invalid priority '{s}', expected normal|urgent|critical"))
}

fn failure_strategy_to_str(s: &ForkFailureStrategy) -> &'static str {
    match s {
        ForkFailureStrategy::DropAndLog => "drop_and_log",
        ForkFailureStrategy::FailSend => "fail_send",
    }
}

fn parse_failure_strategy(s: &Option<String>) -> Result<ForkFailureStrategy, String> {
    match s.as_deref() {
        None | Some("") | Some("drop_and_log") => Ok(ForkFailureStrategy::DropAndLog),
        Some("fail_send") => Ok(ForkFailureStrategy::FailSend),
        Some(other) => Err(format!(
            "invalid on_failure '{other}', expected drop_and_log|fail_send"
        )),
    }
}

fn build_rule_from_create(req: CreateForwardRuleReq) -> Result<Mq9ForwardRule, String> {
    let priorities = req
        .priorities
        .iter()
        .map(|p| parse_priority(p))
        .collect::<Result<Vec<_>, _>>()?;
    let on_failure = parse_failure_strategy(&req.on_failure)?;
    let now = now_second();
    Ok(Mq9ForwardRule {
        tenant: req.tenant,
        rule_name: req.rule_name,
        matcher: Mq9ForwardMatcher {
            mail_address_prefixes: req.mail_address_prefixes,
            any_tags: req.any_tags,
            priorities,
            sender_prefixes: req.sender_prefixes,
        },
        target: Mq9ForwardTarget {
            topic_name: req.topic_name,
            keep_headers: req.keep_headers,
            on_failure,
        },
        etl_rule: req.etl_rule,
        enabled: req.enabled,
        create_time: now,
        update_time: now,
    })
}

fn build_rule_from_update(
    req: UpdateForwardRuleReq,
    create_time: u64,
) -> Result<Mq9ForwardRule, String> {
    let priorities = req
        .priorities
        .iter()
        .map(|p| parse_priority(p))
        .collect::<Result<Vec<_>, _>>()?;
    let on_failure = parse_failure_strategy(&req.on_failure)?;
    Ok(Mq9ForwardRule {
        tenant: req.tenant,
        rule_name: req.rule_name,
        matcher: Mq9ForwardMatcher {
            mail_address_prefixes: req.mail_address_prefixes,
            any_tags: req.any_tags,
            priorities,
            sender_prefixes: req.sender_prefixes,
        },
        target: Mq9ForwardTarget {
            topic_name: req.topic_name,
            keep_headers: req.keep_headers,
            on_failure,
        },
        etl_rule: req.etl_rule,
        enabled: req.enabled,
        create_time,
        update_time: now_second(),
    })
}

pub async fn forward_rule_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ForwardRuleListReq>,
) -> String {
    if state.nats_context.is_none() {
        return error_response("nats-broker is not running".to_string());
    }

    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        None,
        None,
        None,
    );

    let storage = Mq9ForwardRuleStorage::new(state.client_pool.clone());
    let rules = match storage.list(params.tenant.as_deref().unwrap_or(""), "").await {
        Ok(rs) => rs,
        Err(e) => return error_response(e.to_string()),
    };

    let topic_filter = params.topic_name.as_deref();
    let rule_name_filter = params.rule_name.as_deref();
    let enabled_filter = params.enabled;

    let rows: Vec<ForwardRuleListRow> = rules
        .iter()
        .filter(|r| {
            if let Some(t) = topic_filter {
                if !r.target.topic_name.contains(t) {
                    return false;
                }
            }
            if let Some(n) = rule_name_filter {
                if !r.rule_name.contains(n) {
                    return false;
                }
            }
            if let Some(e) = enabled_filter {
                if r.enabled != e {
                    return false;
                }
            }
            true
        })
        .map(from_rule)
        .collect();

    let sorted = apply_sorting(rows, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

pub async fn forward_rule_detail(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ForwardRuleDetailReq>,
) -> String {
    if state.nats_context.is_none() {
        return error_response("nats-broker is not running".to_string());
    }

    let storage = Mq9ForwardRuleStorage::new(state.client_pool.clone());
    let mut rules = match storage.list(&params.tenant, &params.rule_name).await {
        Ok(rs) => rs,
        Err(e) => return error_response(e.to_string()),
    };

    match rules.pop() {
        Some(rule) => success_response(from_rule(&rule)),
        None => error_response(format!(
            "forward rule '{}/{}' does not exist",
            params.tenant, params.rule_name
        )),
    }
}

pub async fn forward_rule_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateForwardRuleReq>,
) -> String {
    if state.nats_context.is_none() {
        return error_response("nats-broker is not running".to_string());
    }

    let rule = match build_rule_from_create(params) {
        Ok(r) => r,
        Err(e) => return error_response(e),
    };

    let storage = Mq9ForwardRuleStorage::new(state.client_pool.clone());
    if let Err(e) = storage.create(&rule).await {
        return error_response(e.to_string());
    }
    success_response("success")
}

pub async fn forward_rule_update(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<UpdateForwardRuleReq>,
) -> String {
    if state.nats_context.is_none() {
        return error_response("nats-broker is not running".to_string());
    }

    let storage = Mq9ForwardRuleStorage::new(state.client_pool.clone());
    let existing = match storage.list(&params.tenant, &params.rule_name).await {
        Ok(mut rs) => rs.pop(),
        Err(e) => return error_response(e.to_string()),
    };

    let create_time = match existing {
        Some(r) => r.create_time,
        None => {
            return error_response(format!(
                "forward rule '{}/{}' does not exist",
                params.tenant, params.rule_name
            ))
        }
    };

    let rule = match build_rule_from_update(params, create_time) {
        Ok(r) => r,
        Err(e) => return error_response(e),
    };

    if let Err(e) = storage.update(&rule).await {
        return error_response(e.to_string());
    }
    success_response("success")
}

pub async fn forward_rule_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteForwardRuleReq>,
) -> String {
    if state.nats_context.is_none() {
        return error_response("nats-broker is not running".to_string());
    }

    let storage = Mq9ForwardRuleStorage::new(state.client_pool.clone());
    if let Err(e) = storage.delete(&params.tenant, &params.rule_name).await {
        return error_response(e.to_string());
    }
    success_response("success")
}
