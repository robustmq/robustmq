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

    use grpc_clients::{
        meta::inner::call::{create_schema, delete_schema, list_schema, update_schema},
        pool::ClientPool,
    };
    use metadata_struct::schema::{SchemaData, SchemaType};
    use protocol::meta::meta_service_inner::{
        CreateSchemaRequest, DeleteSchemaRequest, ListSchemaRequest, UpdateSchemaRequest,
    };

    use crate::common::get_placement_addr;

    fn check_schema_equal(left: &SchemaData, right: &SchemaData) {
        assert_eq!(left.cluster_name, right.cluster_name);
        assert_eq!(left.name, right.name);
        assert_eq!(left.schema_type, right.schema_type);
        assert_eq!(left.schema, right.schema);
        assert_eq!(left.desc, right.desc);
    }

    #[tokio::test]
    async fn schema_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];

        let cluster_name = "test_cluster".to_string();
        let schema_name = "test_schema".to_string();

        let mut schema_data = SchemaData {
            cluster_name: cluster_name.clone(),
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
            cluster_name: cluster_name.clone(),
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
            cluster_name: cluster_name.clone(),
            schema_name: "".to_string(),
        };

        match list_schema(&client_pool, &addrs, list_request.clone()).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 1);
                let schema = SchemaData::decode(reply.schemas.first().unwrap()).unwrap();

                check_schema_equal(&schema, &schema_data);
            }

            Err(e) => {
                panic!("list schema failed: {e}");
            }
        }

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
            cluster_name: cluster_name.clone(),
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
        match list_schema(&client_pool, &addrs, list_request.clone()).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 1);
                let schema = SchemaData::decode(reply.schemas.first().unwrap()).unwrap();

                check_schema_equal(&schema, &schema_data);
            }

            Err(e) => {
                panic!("list schema failed: {e}");
            }
        }

        // delete schema
        let delete_request = DeleteSchemaRequest {
            cluster_name: cluster_name.clone(),
            schema_name: schema_name.clone(),
        };

        match delete_schema(&client_pool, &addrs, delete_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("delete schema failed: {e}");
            }
        }

        match list_schema(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 0);
            }

            Err(e) => {
                panic!("list schema failed: {e}");
            }
        }
    }
}
