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

use super::error::MetaServiceError;
use crate::{
    controller::mqtt::call_broker::{
        update_cache_by_add_schema, update_cache_by_add_schema_bind, update_cache_by_delete_schema,
        update_cache_by_delete_schema_bind, MQTTInnerCallManager,
    },
    raft::route::{
        apply::RaftMachineManager,
        data::{StorageData, StorageDataType},
    },
    storage::placement::schema::SchemaStorage,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use prost::Message;
use prost_validate::Result;
use protocol::meta::meta_service_inner::{
    BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListBindSchemaRequest,
    ListSchemaRequest, UnBindSchemaRequest, UpdateSchemaRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

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
    raft_machine_apply: &Arc<RaftMachineManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateSchemaRequest,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<(), MetaServiceError> {
    if req.cluster_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }

    if req.schema_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }

    if req.schema.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema".to_string(),
        ));
    }
    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());
    if let Some(_data) = schema_storage.get(&req.cluster_name, &req.schema_name)? {
        Err(MetaServiceError::SchemaAlreadyExist(
            "schema_name".to_string(),
        ))
    } else {
        let data = StorageData::new(
            StorageDataType::SchemaSet,
            CreateSchemaRequest::encode_to_vec(req),
        );
        raft_machine_apply.client_write(data).await?;

        let schema = SchemaData::decode(&req.schema)?;
        update_cache_by_add_schema(&req.cluster_name, call_manager, client_pool, schema).await?;
        Ok(())
    }
}

pub async fn update_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSchemaRequest,
) -> Result<(), MetaServiceError> {
    let storage = SchemaStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.cluster_name, &req.schema_name)?.is_none() {
        return Err(MetaServiceError::SchemaNotFound(req.schema_name.clone()));
    };

    if req.cluster_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }

    if req.schema_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }

    if req.schema.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaSet,
        UpdateSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    let schema = SchemaData::decode(&req.schema)?;
    update_cache_by_add_schema(&req.cluster_name, call_manager, client_pool, schema).await?;
    Ok(())
}

pub async fn delete_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSchemaRequest,
) -> Result<(), MetaServiceError> {
    let storage = SchemaStorage::new(rocksdb_engine_handler.clone());
    let schema = if let Some(schema) = storage.get(&req.cluster_name, &req.schema_name)? {
        schema
    } else {
        return Err(MetaServiceError::SchemaDoesNotExist(
            req.schema_name.clone(),
        ));
    };
    if req.cluster_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }

    if req.schema_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaDelete,
        DeleteSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    update_cache_by_delete_schema(&req.cluster_name, call_manager, client_pool, schema).await?;
    Ok(())
}

pub async fn list_bind_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListBindSchemaRequest,
) -> Result<Vec<Vec<u8>>, MetaServiceError> {
    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // get schema bind
    if !req.cluster_name.is_empty() && !req.schema_name.is_empty() && !req.resource_name.is_empty()
    {
        if let Some(res) =
            schema_storage.get_bind(&req.cluster_name, &req.schema_name, &req.resource_name)?
        {
            return Ok(vec![res.encode()?]);
        } else {
            return Ok(Vec::new());
        }
    }

    // get schema bind by cluster_name
    if !req.cluster_name.is_empty() && req.schema_name.is_empty() && req.resource_name.is_empty() {
        let results = schema_storage
            .list_bind_by_cluster(&req.cluster_name)?
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(results);
    }

    // get schema bind by resource
    if !req.cluster_name.is_empty() && req.schema_name.is_empty() && !req.resource_name.is_empty() {
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
    raft_machine_apply: &Arc<RaftMachineManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &BindSchemaRequest,
) -> Result<(), MetaServiceError> {
    if req.cluster_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }
    if req.schema_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }
    if req.resource_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "resource_name".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaBindSet,
        BindSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

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
    raft_machine_apply: &Arc<RaftMachineManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UnBindSchemaRequest,
) -> Result<(), MetaServiceError> {
    if req.cluster_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }
    if req.schema_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }
    if req.resource_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            "resource_name".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaBindDelete,
        UnBindSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    let schema_data = SchemaResourceBind {
        cluster_name: req.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    update_cache_by_delete_schema_bind(&req.cluster_name, call_manager, client_pool, schema_data)
        .await?;
    Ok(())
}
