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
use crate::handler::command::NatsProcessContext;
use a2a_types::AgentCard;
use bytes::Bytes;
use mq9_core::protocol::{
    AgentDiscoverReply, AgentRegisterReply, AgentReportReply, AgentUnregisterReply,
};

pub async fn process_agent_register(
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<AgentRegisterReply, NatsBrokerError> {
    Ok(AgentRegisterReply::default())
}

pub async fn process_agent_unregister(
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<AgentUnregisterReply, NatsBrokerError> {
    Ok(AgentUnregisterReply::default())
}

pub async fn process_agent_report(
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<AgentReportReply, NatsBrokerError> {
    Ok(AgentReportReply::default())
}

pub async fn process_agent_discover(
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<AgentDiscoverReply, NatsBrokerError> {
    Ok(AgentDiscoverReply::default())
}
