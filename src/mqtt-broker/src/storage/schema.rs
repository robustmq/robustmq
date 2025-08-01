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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::{placement::inner::call::list_schema, pool::ClientPool};
use metadata_struct::schema::SchemaData;
use protocol::placement_center::placement_center_inner::ListSchemaRequest;
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
            cluster_name: config.cluster_name.clone(),
            schema_name,
        };

        let reply = list_schema(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        let mut results = Vec::new();
        for raw in reply.schemas {
            results.push(serde_json::from_slice::<SchemaData>(raw.as_slice())?);
        }
        Ok(results)
    }
}
