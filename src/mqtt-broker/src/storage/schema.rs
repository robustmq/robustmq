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

use common_base::error::{common::CommonError, ResultCommonError};
use common_config::broker::broker_config;
use grpc_clients::{
    meta::common::call::{
        bind_schema, create_schema, delete_schema, list_bind_schema, list_schema, un_bind_schema,
    },
    pool::ClientPool,
};
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use protocol::meta::meta_service_common::{
    BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListBindSchemaRequest,
    ListSchemaRequest, UnBindSchemaRequest,
};
use std::sync::Arc;

pub struct SchemaStorage {
    client_pool: Arc<ClientPool>,
}

impl SchemaStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        SchemaStorage { client_pool }
    }

    pub async fn list(&self, schema_name: String) -> Result<Vec<SchemaData>, CommonError> {
        let config = broker_config();
        let request = ListSchemaRequest {
            schema_name,
        };

        let reply =
            list_schema(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        let mut results = Vec::new();
        for raw in reply.schemas {
            results.push(SchemaData::decode(&raw)?);
        }
        Ok(results)
    }

    pub async fn create(&self, schema_data: SchemaData) -> ResultCommonError {
        let config = broker_config();
        let request = CreateSchemaRequest {
            schema_name: schema_data.name.clone(),
            schema: schema_data.encode()?,
        };

        create_schema(&self.client_pool, &config.get_meta_service_addr(), request).await?;

        Ok(())
    }

    pub async fn delete(&self, schema_name: String) -> ResultCommonError {
        let config = broker_config();
        let request = DeleteSchemaRequest {
            schema_name,
        };

        delete_schema(&self.client_pool, &config.get_meta_service_addr(), request).await?;

        Ok(())
    }

    pub async fn create_bind(&self, schema_name: &str, resource_name: &str) -> ResultCommonError {
        let config = broker_config();
        let request = BindSchemaRequest {
            schema_name: schema_name.to_string(),
            resource_name: resource_name.to_string(),
        };

        bind_schema(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_bind(&self, schema_name: &str, resource_name: &str) -> ResultCommonError {
        let config = broker_config();
        let request = UnBindSchemaRequest {
            schema_name: schema_name.to_string(),
            resource_name: resource_name.to_string(),
        };

        un_bind_schema(&self.client_pool, &config.get_meta_service_addr(), request).await?;

        Ok(())
    }

    pub async fn list_bind(&self) -> Result<Vec<SchemaResourceBind>, CommonError> {
        let config = broker_config();
        let request = ListBindSchemaRequest {
            schema_name: "".to_string(),
            resource_name: "".to_string(),
        };

        let reply =
            list_bind_schema(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        let mut results = Vec::new();
        for raw in reply.schema_binds {
            results.push(SchemaResourceBind::decode(&raw)?);
        }
        Ok(results)
    }
}
