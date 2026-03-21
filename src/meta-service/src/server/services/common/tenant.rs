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
    send_notify_by_create_tenant, send_notify_by_delete_tenant, send_notify_by_update_tenant,
};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::common::tenant::TenantStorage;
use common_base::tools::now_second;
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::tenant::{Tenant, TenantConfig};
use node_call::NodeCallManager;
use protocol::meta::meta_service_common::{
    CreateTenantReply, CreateTenantRequest, DeleteTenantReply, DeleteTenantRequest,
    ListTenantReply, ListTenantRequest, UpdateTenantReply, UpdateTenantRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

type ListTenantStream =
    Result<Pin<Box<dyn Stream<Item = Result<ListTenantReply, Status>> + Send>>, MetaServiceError>;

pub async fn create_tenant_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateTenantRequest,
) -> Result<CreateTenantReply, MetaServiceError> {
    if TenantStorage::new(rocksdb_engine_handler.clone())
        .get(&req.tenant_name)?
        .is_some()
    {
        return Err(MetaServiceError::CommonError(format!(
            "Tenant {} already exists",
            req.tenant_name
        )));
    }

    let data = StorageData::new(StorageDataType::TenantCreate, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let config = if req.config.is_empty() {
        TenantConfig::default()
    } else {
        TenantConfig::decode(&req.config).unwrap_or_default()
    };
    let tenant = Tenant {
        tenant_name: req.tenant_name.clone(),
        desc: req.desc.clone(),
        config,
        create_time: now_second(),
    };
    send_notify_by_create_tenant(call_manager, tenant).await?;

    Ok(CreateTenantReply {})
}

pub async fn update_tenant_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &UpdateTenantRequest,
) -> Result<UpdateTenantReply, MetaServiceError> {
    let storage = TenantStorage::new(rocksdb_engine_handler.clone());
    let mut tenant = storage.get(&req.tenant_name)?.ok_or_else(|| {
        MetaServiceError::CommonError(format!("Tenant {} not found", req.tenant_name))
    })?;

    tenant.desc = req.desc.clone();
    if !req.config.is_empty() {
        tenant.config = TenantConfig::decode(&req.config)
            .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;
    }

    let data = StorageData::new(StorageDataType::TenantUpdate, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    send_notify_by_update_tenant(call_manager, tenant).await?;

    Ok(UpdateTenantReply {})
}

pub async fn delete_tenant_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteTenantRequest,
) -> Result<DeleteTenantReply, MetaServiceError> {
    let tenant = match TenantStorage::new(rocksdb_engine_handler.clone()).get(&req.tenant_name)? {
        Some(t) => t,
        None => return Ok(DeleteTenantReply {}),
    };

    let data = StorageData::new(StorageDataType::TenantDelete, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    send_notify_by_delete_tenant(call_manager, tenant).await?;

    Ok(DeleteTenantReply {})
}

pub fn list_tenant_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListTenantRequest,
) -> ListTenantStream {
    let storage = TenantStorage::new(rocksdb_engine_handler.clone());

    let tenants: Vec<Vec<u8>> = if req.tenant_name.is_empty() {
        storage
            .list()?
            .into_iter()
            .map(|t| t.encode())
            .collect::<Result<Vec<_>, _>>()?
    } else {
        match storage.get(&req.tenant_name)? {
            Some(t) => vec![t.encode()?],
            None => vec![],
        }
    };

    let output = async_stream::try_stream! {
        for tenant in tenants {
            yield ListTenantReply { tenant };
        }
    };

    Ok(Box::pin(output))
}
