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
    controller::mqtt::call_broker::{
        update_cache_by_add_schema, update_cache_by_add_schema_bind, update_cache_by_delete_schema,
        update_cache_by_delete_schema_bind, MQTTInnerCallManager,
    },
    core::error::MetaServiceError,
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
    storage::placement::schema::SchemaStorage,
};
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use prost_validate::Result;
use protocol::meta::meta_service_common::{
    BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListBindSchemaRequest,
    ListSchemaRequest, UnBindSchemaRequest, UpdateSchemaRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

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
fn validate_schema_fields(
    cluster_name: &str,
    schema_name: &str,
    schema: &[u8],
) -> Result<(), MetaServiceError> {
    validate_non_empty(cluster_name, "cluster_name")?;
    validate_non_empty(schema_name, "schema_name")?;
    if schema.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema".to_string(),
        ));
    }
    Ok(())
}

// Helper: Validate required fields for schema bind operations
fn validate_bind_fields(
    cluster_name: &str,
    schema_name: &str,
    resource_name: &str,
) -> Result<(), MetaServiceError> {
    validate_non_empty(cluster_name, "cluster_name")?;
    validate_non_empty(schema_name, "schema_name")?;
    validate_non_empty(resource_name, "resource_name")?;
    Ok(())
}

// Schema Operations
pub fn list_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSchemaRequest,
) -> Result<Vec<Vec<u8>>, MetaServiceError> {
    if req.cluster_name.is_empty() {
        return Ok(Vec::new());
    }

    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());
    let list = if !req.schema_name.is_empty() {
        if let Some(data) = schema_storage.get(&req.cluster_name, &req.schema_name)? {
            vec![data]
        } else {
            vec![]
        }
    } else {
        schema_storage.list(&req.cluster_name)?
    };

    let results = list
        .into_iter()
        .map(|data| data.encode())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(results)
}

pub async fn create_schema_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateSchemaRequest,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<(), MetaServiceError> {
    validate_schema_fields(&req.cluster_name, &req.schema_name, &req.schema)?;

    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // Check if schema already exists
    if schema_storage
        .get(&req.cluster_name, &req.schema_name)?
        .is_some()
    {
        return Err(MetaServiceError::SchemaAlreadyExist(
            req.schema_name.clone(),
        ));
    }

    let data = StorageData::new(StorageDataType::SchemaSet, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema = SchemaData::decode(&req.schema)?;
    update_cache_by_add_schema(&req.cluster_name, call_manager, client_pool, schema).await?;

    Ok(())
}

pub async fn update_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_schema_fields(&req.cluster_name, &req.schema_name, &req.schema)?;

    let storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // Check if schema exists
    if storage.get(&req.cluster_name, &req.schema_name)?.is_none() {
        return Err(MetaServiceError::SchemaNotFound(req.schema_name.clone()));
    }

    let data = StorageData::new(StorageDataType::SchemaSet, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema = SchemaData::decode(&req.schema)?;
    update_cache_by_add_schema(&req.cluster_name, call_manager, client_pool, schema).await?;

    Ok(())
}

pub async fn delete_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_non_empty(&req.cluster_name, "cluster_name")?;
    validate_non_empty(&req.schema_name, "schema_name")?;

    let storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // Get schema to delete (must exist)
    let schema = storage
        .get(&req.cluster_name, &req.schema_name)?
        .ok_or_else(|| MetaServiceError::SchemaDoesNotExist(req.schema_name.clone()))?;

    let data = StorageData::new(StorageDataType::SchemaDelete, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    update_cache_by_delete_schema(&req.cluster_name, call_manager, client_pool, schema).await?;

    Ok(())
}

// Schema Bind Operations
pub async fn list_bind_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListBindSchemaRequest,
) -> Result<Vec<Vec<u8>>, MetaServiceError> {
    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    let has_cluster = !req.cluster_name.is_empty();
    let has_schema = !req.schema_name.is_empty();
    let has_resource = !req.resource_name.is_empty();

    // Get specific schema bind (all three fields provided)
    if has_cluster && has_schema && has_resource {
        return match schema_storage.get_bind(
            &req.cluster_name,
            &req.schema_name,
            &req.resource_name,
        )? {
            Some(bind) => Ok(vec![bind.encode()?]),
            None => Ok(Vec::new()),
        };
    }

    // List by cluster only
    if has_cluster && !has_schema && !has_resource {
        let results = schema_storage
            .list_bind_by_cluster(&req.cluster_name)?
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(results);
    }

    // List by cluster and resource
    if has_cluster && !has_schema && has_resource {
        let results = schema_storage
            .list_bind_by_resource(&req.cluster_name, &req.resource_name)?
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(results);
    }

    Ok(Vec::new())
}

pub async fn bind_schema_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &BindSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_bind_fields(&req.cluster_name, &req.schema_name, &req.resource_name)?;

    let data = StorageData::new(StorageDataType::SchemaBindSet, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema_data = SchemaResourceBind {
        cluster_name: req.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    update_cache_by_add_schema_bind(&req.cluster_name, call_manager, client_pool, schema_data)
        .await?;

    Ok(())
}

pub async fn un_bind_schema_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UnBindSchemaRequest,
) -> Result<(), MetaServiceError> {
    validate_bind_fields(&req.cluster_name, &req.schema_name, &req.resource_name)?;

    let data = StorageData::new(StorageDataType::SchemaBindDelete, encode_to_bytes(req));
    raft_manager.write_metadata(data).await?;

    let schema_data = SchemaResourceBind {
        cluster_name: req.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    update_cache_by_delete_schema_bind(&req.cluster_name, call_manager, client_pool, schema_data)
        .await?;

    Ok(())
}
