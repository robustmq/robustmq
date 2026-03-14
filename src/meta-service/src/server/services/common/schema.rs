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
    core::error::MetaServiceError,
    core::notify::{
        send_notify_by_add_schema, send_notify_by_add_schema_bind, send_notify_by_delete_schema,
        send_notify_by_delete_schema_bind,
    },
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
    storage::common::schema::SchemaStorage,
};
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use node_call::NodeCallManager;
use prost_validate::Result;
use protocol::meta::meta_service_common::{
    BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListBindSchemaReply,
    ListBindSchemaRequest, ListSchemaReply, ListSchemaRequest, UnBindSchemaRequest,
    UpdateSchemaRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

type ListSchemaStream = std::result::Result<
    Pin<Box<dyn Stream<Item = std::result::Result<ListSchemaReply, Status>> + Send>>,
    MetaServiceError,
>;

type ListBindSchemaStream = std::result::Result<
    Pin<Box<dyn Stream<Item = std::result::Result<ListBindSchemaReply, Status>> + Send>>,
    MetaServiceError,
>;

// Helper: Validate non-empty field
fn validate_non_empty(value: &str, field_name: &str) -> Result<(), MetaServiceError> {
    if value.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            field_name.to_string(),
        ));
    }
    Ok(())
}

// Helper: Validate required fields for schema operations
fn validate_schema_fields(schema_name: &str, schema: &[u8]) -> Result<(), MetaServiceError> {
    validate_non_empty(schema_name, "schema_name")?;
    if schema.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema".to_string(),
        ));
    }
    Ok(())
}

// Helper: Validate required fields for schema bind operations
fn validate_bind_fields(schema_name: &str, resource_name: &str) -> Result<(), MetaServiceError> {
    validate_non_empty(schema_name, "schema_name")?;
    validate_non_empty(resource_name, "resource_name")?;
    Ok(())
}

// Schema Operations
pub fn list_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSchemaRequest,
) -> ListSchemaStream {
    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());
    let list = if !req.schema_name.is_empty() {
        if let Some(data) = schema_storage.get(&req.tenant, &req.schema_name)? {
            vec![data]
        } else {
            vec![]
        }
    } else if !req.tenant.is_empty() {
        schema_storage.list_by_tenant(&req.tenant)?
    } else {
        schema_storage.list()?
    };

    let schemas = list
        .into_iter()
        .map(|data| data.encode())
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let output = async_stream::try_stream! {
        for schema in schemas {
            yield ListSchemaReply { schema };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_schema_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    req: &CreateSchemaRequest,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<(), MetaServiceError> {
    validate_schema_fields(&req.schema_name, &req.schema)?;

    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // Check if schema already exists
    if schema_storage.get(&req.tenant, &req.schema_name)?.is_some() {
        return Err(MetaServiceError::SchemaAlreadyExist(
            req.schema_name.clone(),
        ));
    }

    let data = StorageData::new(StorageDataType::SchemaSet, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema = SchemaData::decode(&req.schema)?;
    send_notify_by_add_schema(call_manager, schema).await?;

    Ok(())
}

pub async fn update_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    req: &UpdateSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_schema_fields(&req.schema_name, &req.schema)?;

    let storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // Check if schema exists
    if storage.get(&req.tenant, &req.schema_name)?.is_none() {
        return Err(MetaServiceError::SchemaNotFound(req.schema_name.clone()));
    }

    let data = StorageData::new(StorageDataType::SchemaSet, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema = SchemaData::decode(&req.schema)?;
    send_notify_by_add_schema(call_manager, schema).await?;

    Ok(())
}

pub async fn delete_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    req: &DeleteSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_non_empty(&req.schema_name, "schema_name")?;

    let storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // Get schema to delete (must exist)
    let schema = storage
        .get(&req.tenant, &req.schema_name)?
        .ok_or_else(|| MetaServiceError::SchemaDoesNotExist(req.schema_name.clone()))?;

    let data = StorageData::new(StorageDataType::SchemaDelete, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    send_notify_by_delete_schema(call_manager, schema).await?;

    Ok(())
}

// Schema Bind Operations
pub async fn list_bind_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListBindSchemaRequest,
) -> ListBindSchemaStream {
    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    let has_schema = !req.schema_name.is_empty();
    let has_resource = !req.resource_name.is_empty();
    let has_tenant = !req.tenant.is_empty();

    let schema_binds: Vec<Vec<u8>> = if has_schema && has_resource && has_tenant {
        match schema_storage.get_bind(&req.tenant, &req.resource_name, &req.schema_name)? {
            Some(bind) => vec![bind.encode()?],
            None => Vec::new(),
        }
    } else if has_tenant && has_resource {
        schema_storage
            .list_bind_by_resource(&req.tenant, &req.resource_name)?
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<std::result::Result<Vec<_>, _>>()?
    } else if has_tenant {
        schema_storage
            .list_bind_by_tenant(&req.tenant)?
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        schema_storage
            .list_bind()?
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<std::result::Result<Vec<_>, _>>()?
    };

    let output = async_stream::try_stream! {
        for schema_bind in schema_binds {
            yield ListBindSchemaReply { schema_bind };
        }
    };

    Ok(Box::pin(output))
}

pub async fn bind_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    req: &BindSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_bind_fields(&req.schema_name, &req.resource_name)?;

    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());
    if schema_storage.get(&req.tenant, &req.schema_name)?.is_none() {
        return Err(MetaServiceError::SchemaDoesNotExist(
            req.schema_name.clone(),
        ));
    }

    let data = StorageData::new(StorageDataType::SchemaBindSet, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema_data = SchemaResourceBind {
        tenant: req.tenant.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };
    send_notify_by_add_schema_bind(call_manager, schema_data).await?;

    Ok(())
}

pub async fn un_bind_schema_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    req: &UnBindSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_bind_fields(&req.schema_name, &req.resource_name)?;

    let data = StorageData::new(StorageDataType::SchemaBindDelete, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema_data = SchemaResourceBind {
        tenant: req.tenant.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };
    send_notify_by_delete_schema_bind(call_manager, schema_data).await?;

    Ok(())
}
