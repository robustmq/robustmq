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
use crate::core::notify::{
    send_notify_by_create_mq9_forward_rule, send_notify_by_delete_mq9_forward_rule,
    send_notify_by_update_mq9_forward_rule,
};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mq9::forward_rule::Mq9ForwardRuleStorage;
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::mq9::forward_rule::Mq9ForwardRule;
use node_call::NodeCallManager;
use protocol::meta::meta_service_mq9::{
    CreateForwardRuleReply, CreateForwardRuleRequest, DeleteForwardRuleReply,
    DeleteForwardRuleRequest, ListForwardRuleReply, ListForwardRuleRequest, UpdateForwardRuleReply,
    UpdateForwardRuleRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

pub type ListForwardRuleStream = Result<
    Pin<Box<dyn Stream<Item = Result<ListForwardRuleReply, Status>> + Send>>,
    MetaServiceError,
>;

pub fn list_forward_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListForwardRuleRequest,
) -> ListForwardRuleStream {
    let storage = Mq9ForwardRuleStorage::new(rocksdb_engine_handler.clone());

    let rules: Vec<Mq9ForwardRule> = if !req.tenant.is_empty() && !req.rule_name.is_empty() {
        match storage.get(&req.tenant, &req.rule_name)? {
            Some(rule) => vec![rule],
            None => vec![],
        }
    } else if !req.tenant.is_empty() {
        storage.list_by_tenant(&req.tenant)?
    } else {
        storage.list()?
    };

    let output = async_stream::try_stream! {
        for rule in rules {
            yield ListForwardRuleReply { rule: rule.encode()? };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_forward_rule_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateForwardRuleRequest,
) -> Result<CreateForwardRuleReply, MetaServiceError> {
    let rule = Mq9ForwardRule::decode(&req.content)?;

    if rule.tenant != req.tenant || rule.rule_name != req.rule_name {
        return Err(MetaServiceError::CommonError(format!(
            "forward rule body tenant/rule_name ({}, {}) does not match request ({}, {})",
            rule.tenant, rule.rule_name, req.tenant, req.rule_name
        )));
    }

    let storage = Mq9ForwardRuleStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.tenant, &req.rule_name)?.is_some() {
        return Err(MetaServiceError::CommonError(format!(
            "mq9 forward rule '{}/{}' already exists",
            req.tenant, req.rule_name
        )));
    }

    let data = StorageData::new(StorageDataType::Mq9CreateForwardRule, encode_to_bytes(req));
    raft_manager.write_data(&req.tenant, data).await?;

    send_notify_by_create_mq9_forward_rule(call_manager, rule).await?;

    Ok(CreateForwardRuleReply {})
}

pub async fn update_forward_rule_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &UpdateForwardRuleRequest,
) -> Result<UpdateForwardRuleReply, MetaServiceError> {
    let rule = Mq9ForwardRule::decode(&req.content)?;

    if rule.tenant != req.tenant || rule.rule_name != req.rule_name {
        return Err(MetaServiceError::CommonError(format!(
            "forward rule body tenant/rule_name ({}, {}) does not match request ({}, {})",
            rule.tenant, rule.rule_name, req.tenant, req.rule_name
        )));
    }

    let storage = Mq9ForwardRuleStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.tenant, &req.rule_name)?.is_none() {
        return Err(MetaServiceError::CommonError(format!(
            "mq9 forward rule '{}/{}' does not exist",
            req.tenant, req.rule_name
        )));
    }

    let data = StorageData::new(StorageDataType::Mq9UpdateForwardRule, encode_to_bytes(req));
    raft_manager.write_data(&req.tenant, data).await?;

    send_notify_by_update_mq9_forward_rule(call_manager, rule).await?;

    Ok(UpdateForwardRuleReply {})
}

pub async fn delete_forward_rule_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteForwardRuleRequest,
) -> Result<DeleteForwardRuleReply, MetaServiceError> {
    let storage = Mq9ForwardRuleStorage::new(rocksdb_engine_handler.clone());
    if let Some(rule) = storage.get(&req.tenant, &req.rule_name)? {
        let data = StorageData::new(StorageDataType::Mq9DeleteForwardRule, encode_to_bytes(req));
        raft_manager.write_data(&req.tenant, data).await?;
        send_notify_by_delete_mq9_forward_rule(call_manager, rule).await?;
    }
    Ok(DeleteForwardRuleReply {})
}
