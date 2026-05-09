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
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::storage::agent::Mq9AgentStorage;
use a2a_types::AgentCard;
use bytes::Bytes;
use common_base::tools::now_second;
use metadata_struct::mq9::agent::MQ9Agent;
use mq9_core::protocol::{
    AgentDiscoverReply, AgentRegisterReply, AgentRegisterReq, AgentReportReply,
    AgentUnregisterReply,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AgentUnregisterReq {
    pub name: String,
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
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<AgentReportReply, NatsBrokerError> {
    Ok(AgentReportReply::default())
}

pub async fn process_agent_discover(
    ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<AgentDiscoverReply, NatsBrokerError> {
    let tenant = get_tenant();
    let agents = Mq9AgentStorage::new(ctx.client_pool.clone())
        .list(&tenant)
        .await?;

    let agent_values: Vec<serde_json::Value> = agents
        .into_iter()
        .filter_map(|a| serde_json::from_str(&a.agent_info).ok())
        .collect();

    Ok(AgentDiscoverReply {
        error: String::new(),
        agents: agent_values,
    })
}
