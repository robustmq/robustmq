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

use crate::core::error::MetaServiceError;
use crate::core::notify::{send_notify_by_create_mq9_agent, send_notify_by_delete_mq9_agent};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mq9::agent::Mq9AgentStorage;
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::mq9::agent::MQ9Agent;
use node_call::NodeCallManager;
use protocol::meta::meta_service_mq9::{
    AgentSearchItem, CreateAgentReply, CreateAgentRequest, DeleteAgentReply, DeleteAgentRequest,
    ListAgentReply, ListAgentRequest, SearchAgentReply, SearchAgentRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

pub type ListAgentStream =
    Result<Pin<Box<dyn Stream<Item = Result<ListAgentReply, Status>> + Send>>, MetaServiceError>;

pub fn list_agent_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListAgentRequest,
) -> ListAgentStream {
    let storage = Mq9AgentStorage::new(rocksdb_engine_handler.clone());
    let agents: Vec<MQ9Agent> = if !req.tenant.is_empty() {
        storage.list_by_tenant(&req.tenant)?
    } else {
        storage.list()?
    };

    let output = async_stream::try_stream! {
        for agent in agents {
            yield ListAgentReply { agent: agent.encode()? };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_agent_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateAgentRequest,
) -> Result<CreateAgentReply, MetaServiceError> {
    let agent = MQ9Agent::decode(&req.content)?;

    let storage = Mq9AgentStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.tenant, &agent.name)?.is_some() {
        return Ok(CreateAgentReply {});
    }

    let data = StorageData::new(StorageDataType::Mq9CreateAgent, encode_to_bytes(req));
    raft_manager.write_data(&req.tenant, data).await?;

    send_notify_by_create_mq9_agent(call_manager, agent).await?;

    Ok(CreateAgentReply {})
}

pub async fn delete_agent_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteAgentRequest,
) -> Result<DeleteAgentReply, MetaServiceError> {
    let storage = Mq9AgentStorage::new(rocksdb_engine_handler.clone());
    if let Some(agent) = storage.get(&req.tenant, &req.name)? {
        let data = StorageData::new(StorageDataType::Mq9DeleteAgent, encode_to_bytes(req));
        raft_manager.write_data(&req.tenant, data).await?;
        send_notify_by_delete_mq9_agent(call_manager, agent).await?;
    }
    Ok(DeleteAgentReply {})
}

pub async fn search_agent_by_req(
    req: &SearchAgentRequest,
) -> Result<SearchAgentReply, MetaServiceError> {
    let limit = if req.limit == 0 {
        10
    } else {
        req.limit as usize
    };
    let offset = req.offset as usize;
    let tenant = if req.tenant.is_empty() {
        None
    } else {
        Some(req.tenant.as_str())
    };

    let results = if !req.semantic.is_empty() {
        let vector = llm_engine::embedding::fastembed::embed(&req.semantic)
            .await
            .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;
        search_engine::agent::search_agents_by_vector(vector, limit, offset, tenant)
            .await
            .map_err(|e| MetaServiceError::CommonError(e.to_string()))?
    } else if !req.text.is_empty() {
        search_engine::agent::search_agents_by_text(&req.text, limit, offset)
            .await
            .map_err(|e| MetaServiceError::CommonError(e.to_string()))?
    } else {
        vec![]
    };

    let items = results
        .into_iter()
        .map(|r| AgentSearchItem {
            agent_id: r.agent_id,
            name: r.name,
            description: r.description,
            agent_info: r.agent_info,
        })
        .collect();

    Ok(SearchAgentReply { items })
}
