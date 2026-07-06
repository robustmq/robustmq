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

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use common_base::uuid::unique_id;
    use grpc_clients::{
        meta::common::call::{create_schema, delete_schema, list_schema, update_schema},
        pool::ClientPool,
    };
    use metadata_struct::schema::{SchemaData, SchemaType};
    use protocol::meta::meta_service_common::{
        CreateSchemaRequest, DeleteSchemaRequest, ListSchemaRequest, UpdateSchemaRequest,
    };

    use crate::common::{get_placement_addr, wait_until};

    fn schema_matches(left: &SchemaData, right: &SchemaData) -> bool {
        left.name == right.name
            && left.schema_type == right.schema_type
            && left.schema == right.schema
            && left.desc == right.desc
    }

    #[tokio::test]
    async fn schema_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];

        let schema_name = unique_id();

        let tenant = "default".to_string();

        let mut schema_data = SchemaData {
            tenant: tenant.clone(),
            name: schema_name.clone(),
            schema_type: SchemaType::JSON,
            schema: r#"{
                "type":"object",
                "properties":{
                    "name":{
                        "type": "string"
                    },
                    "age":{
                        "type": "integer", "minimum": 0
                    }
                },
                "required":["name"]
            }"#
            .to_string(),
            desc: "Old schema".to_string(),
        };

        let create_request = CreateSchemaRequest {
            tenant: tenant.clone(),
            schema_name: schema_name.clone(),
            schema: schema_data.encode().unwrap(),
        };

        match create_schema(&client_pool, &addrs, create_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("create schema failed: {e}");
            }
        }

        let list_request = ListSchemaRequest {
            tenant: tenant.clone(),
            schema_name: schema_name.clone(),
        };

        let ok = wait_until(|| async {
            let mut stream = match list_schema(&client_pool, &addrs, list_request.clone()).await {
                Ok(s) => s,
                Err(_) => return false,
            };
            let mut schemas = Vec::new();
            while let Ok(Some(reply)) = stream.message().await {
                if let Ok(sd) = SchemaData::decode(&reply.schema) {
                    schemas.push(sd);
                }
            }
            schemas.len() == 1 && schema_matches(&schemas[0], &schema_data)
        })
        .await;
        assert!(ok, "created schema {schema_name} not visible");

        // update schema
        schema_data.schema_type = SchemaType::AVRO;
        schema_data.schema = r#"{
            "type": "record",
            "name": "test",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "age", "type": "int"}
            ]
        }"#
        .to_string();
        schema_data.desc = "New schema".to_string();

        let update_request = UpdateSchemaRequest {
            tenant: tenant.clone(),
            schema_name: schema_name.clone(),
            schema: schema_data.encode().unwrap(),
        };

        match update_schema(&client_pool, &addrs, update_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("update schema failed: {e}");
            }
        }

        // check the schema we just updated
        let ok = wait_until(|| async {
            let mut stream = match list_schema(&client_pool, &addrs, list_request.clone()).await {
                Ok(s) => s,
                Err(_) => return false,
            };
            let mut schemas = Vec::new();
            while let Ok(Some(reply)) = stream.message().await {
                if let Ok(sd) = SchemaData::decode(&reply.schema) {
                    schemas.push(sd);
                }
            }
            schemas.len() == 1 && schema_matches(&schemas[0], &schema_data)
        })
        .await;
        assert!(ok, "updated schema {schema_name} not visible");

        // delete schema
        let delete_request = DeleteSchemaRequest {
            tenant: tenant.clone(),
            schema_name: schema_name.clone(),
        };

        match delete_schema(&client_pool, &addrs, delete_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("delete schema failed: {e}");
            }
        }

        let ok = wait_until(|| async {
            let mut stream = match list_schema(&client_pool, &addrs, list_request.clone()).await {
                Ok(s) => s,
                Err(_) => return false,
            };
            let mut count = 0;
            while let Ok(Some(_reply)) = stream.message().await {
                count += 1;
            }
            count == 0
        })
        .await;
        assert!(ok, "deleted schema {schema_name} still visible");
    }
}
