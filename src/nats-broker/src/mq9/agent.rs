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

use crate::core::error::NatsBrokerError;
use crate::core::subject::try_get_or_init_subject;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::storage::agent::Mq9AgentStorage;
use crate::storage::message::MessageStorage;
use a2a_types::AgentCard;
use broker_core::inner_topic::AGENT_REPORT_INFO_TOPIC;
use bytes::Bytes;
use common_base::tools::now_second;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::mq9::agent::MQ9Agent;
use mq9_core::protocol::{
    AgentDiscoverReply, AgentDiscoverReq, AgentRegisterReply, AgentRegisterReq, AgentReportReply,
    AgentReportReq, AgentUnregisterReply,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AgentUnregisterReq {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct AgentReportRecord {
    pub name: String,
    pub report_info: String,
    pub report_time: u64,
}

pub async fn process_agent_register(
    ctx: &NatsProcessContext,
    payload: &Bytes,
) -> Result<AgentRegisterReply, NatsBrokerError> {
    if payload.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "payload must not be empty".to_string(),
        ));
    }
    let (name, agent_info) = match serde_json::from_slice::<AgentCard>(payload) {
        Ok(card) => {
            let name = card.name.clone();
            let info = serde_json::to_string(&card)?;
            (name, info)
        }
        Err(_) => {
            let req: AgentRegisterReq = serde_json::from_slice(payload).map_err(|e| {
                NatsBrokerError::CommonError(format!("invalid AGENT.REGISTER payload: {}", e))
            })?;
            (req.name, req.payload)
        }
    };

    if agent_info.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "agent_info must not be empty".to_string(),
        ));
    }

    let tenant = get_tenant();
    let agent = MQ9Agent {
        tenant,
        name,
        agent_info,
        create_time: now_second(),
    };

    Mq9AgentStorage::new(ctx.client_pool.clone())
        .create(&agent)
        .await?;

    Ok(AgentRegisterReply {
        error: String::new(),
    })
}

pub async fn process_agent_unregister(
    ctx: &NatsProcessContext,
    payload: &Bytes,
) -> Result<AgentUnregisterReply, NatsBrokerError> {
    let req: AgentUnregisterReq = serde_json::from_slice(payload).map_err(|e| {
        NatsBrokerError::CommonError(format!("invalid AGENT.UNREGISTER payload: {}", e))
    })?;

    if req.name.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "agent name must not be empty".to_string(),
        ));
    }

    let tenant = get_tenant();

    Mq9AgentStorage::new(ctx.client_pool.clone())
        .delete(&tenant, &req.name)
        .await?;

    Ok(AgentUnregisterReply {
        error: String::new(),
    })
}

pub async fn process_agent_report(
    ctx: &NatsProcessContext,
    payload: &Bytes,
) -> Result<AgentReportReply, NatsBrokerError> {
    let req: AgentReportReq = serde_json::from_slice(payload).map_err(|e| {
        NatsBrokerError::CommonError(format!("invalid AGENT.REPORT payload: {}", e))
    })?;

    if req.name.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "agent name must not be empty".to_string(),
        ));
    }

    let tenant = get_tenant();

    let record = AgentReportRecord {
        name: req.name.clone(),
        report_info: req.report_info.unwrap_or_default(),
        report_time: now_second(),
    };
    let payload_str = serde_json::to_string(&record)?;

    try_get_or_init_subject(
        &ctx.cache_manager,
        &ctx.storage_driver_manager,
        &ctx.client_pool,
        &ctx.subscribe_manager,
        &tenant,
        AGENT_REPORT_INFO_TOPIC,
        true,
    )
    .await?;

    let write_record = AdapterWriteRecord::new(AGENT_REPORT_INFO_TOPIC.to_string(), payload_str)
        .with_key(req.name.clone());

    MessageStorage::new(ctx.storage_driver_manager.clone())
        .write(&tenant, AGENT_REPORT_INFO_TOPIC, vec![write_record])
        .await?;

    Ok(AgentReportReply {
        error: String::new(),
    })
}

pub async fn process_agent_discover(
    ctx: &NatsProcessContext,
    payload: &Bytes,
) -> Result<AgentDiscoverReply, NatsBrokerError> {
    let req: AgentDiscoverReq = if payload.is_empty() {
        AgentDiscoverReq::default()
    } else {
        serde_json::from_slice(payload).map_err(|e| {
            NatsBrokerError::CommonError(format!("invalid AGENT.DISCOVER payload: {}", e))
        })?
    };

    let tenant = get_tenant();
    let limit = req.limit.unwrap_or(20);
    let page = req.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let storage = Mq9AgentStorage::new(ctx.client_pool.clone());

    let agents = if let Some(query) = req.semantic.as_deref().filter(|q| !q.is_empty()) {
        storage
            .search_by_semantic(&tenant, query, limit, offset)
            .await?
    } else if let Some(query) = req.text.as_deref().filter(|q| !q.is_empty()) {
        storage
            .search_by_text(&tenant, query, limit, offset)
            .await?
    } else {
        let list = storage.list(&tenant).await?;
        list.into_iter()
            .map(|a| {
                serde_json::json!({
                    "name": a.name,
                    "agent_info": a.agent_info,
                })
            })
            .collect()
    };

    Ok(AgentDiscoverReply {
        error: String::new(),
        agents,
    })
}
