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

use super::error::PlacementCenterError;
use crate::{
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::placement::schema::SchemaStorage,
};
use prost::Message;
use prost_validate::Result;
use protocol::placement_center::placement_center_inner::{
    BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListBindSchemaRequest,
    ListSchemaRequest, UnBindSchemaRequest, UpdateSchemaRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

pub fn list_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSchemaRequest,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
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

    let mut results = Vec::new();
    for data in list {
        results.push(data.encode());
    }
    Ok(results)
}

pub async fn create_schema_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &CreateSchemaRequest,
) -> Result<(), PlacementCenterError> {
    if req.cluster_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }

    if req.schema_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }

    if req.schema.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "schema".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaSet,
        CreateSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    Ok(())
}

pub async fn update_schema_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &UpdateSchemaRequest,
) -> Result<(), PlacementCenterError> {
    if req.cluster_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }

    if req.schema_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }

    if req.schema.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "schema".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaSet,
        UpdateSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    Ok(())
}

pub async fn delete_schema_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &DeleteSchemaRequest,
) -> Result<(), PlacementCenterError> {
    if req.cluster_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }

    if req.schema_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaDelete,
        DeleteSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    Ok(())
}

pub async fn list_bind_schema_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListBindSchemaRequest,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let schema_storage = SchemaStorage::new(rocksdb_engine_handler.clone());

    // get schema bind
    if !req.cluster_name.is_empty() && !req.schema_name.is_empty() && !req.resource_name.is_empty()
    {
        if let Some(res) =
            schema_storage.get_bind(&req.cluster_name, &req.schema_name, &req.resource_name)?
        {
            return Ok(vec![res.encode()]);
        } else {
            return Ok(Vec::new());
        }
    }

    // get schema bind by cluster_name
    if !req.cluster_name.is_empty() && req.schema_name.is_empty() && req.resource_name.is_empty() {
        let mut results = Vec::new();
        for raw in schema_storage.list_bind_by_cluster(&req.cluster_name)? {
            results.push(raw.encode());
        }
        return Ok(results);
    }

    // get schema bind by resource
    if !req.cluster_name.is_empty() && req.schema_name.is_empty() && !req.resource_name.is_empty() {
        let mut results = Vec::new();
        for raw in schema_storage.list_bind_by_resource(&req.cluster_name, &req.resource_name)? {
            results.push(raw.encode());
        }
        return Ok(results);
    }

    Ok(Vec::new())
}

pub async fn bind_schema_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &BindSchemaRequest,
) -> Result<(), PlacementCenterError> {
    if req.cluster_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }
    if req.schema_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }
    if req.resource_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "resource_name".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaBindSet,
        BindSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    Ok(())
}

pub async fn un_bind_schema_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &UnBindSchemaRequest,
) -> Result<(), PlacementCenterError> {
    if req.cluster_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "cluster_name".to_string(),
        ));
    }
    if req.schema_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "schema_name".to_string(),
        ));
    }
    if req.resource_name.is_empty() {
        return Err(PlacementCenterError::RequestParamsNotEmpty(
            "resource_name".to_string(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::SchemaBindDelete,
        UnBindSchemaRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;

    Ok(())
}
