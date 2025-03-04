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
        placement::inner::call::{create_schema, list_schema},
        pool::ClientPool,
    };
    use metadata_struct::schema::{SchemaData, SchemaType};
    use protocol::placement_center::placement_center_inner::{
        CreateSchemaRequest, ListSchemaRequest,
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

        let schema_data = SchemaData {
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
            desc: "".to_string(),
        };

        let create_request = CreateSchemaRequest {
            cluster_name: cluster_name.clone(),
            schema_name: schema_name.clone(),
            schema: serde_json::to_vec(&schema_data).unwrap(),
        };

        match create_schema(&client_pool, &addrs, create_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("create schema failed: {}", e);
            }
        }

        let list_request = ListSchemaRequest {
            cluster_name: cluster_name.clone(),
            schema_name: "".to_string(),
        };

        match list_schema(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 1);
                let schema =
                    serde_json::from_slice::<SchemaData>(reply.schemas.get(0).unwrap()).unwrap();

                check_schema_equal(&schema, &schema_data);
            }

            Err(e) => {
                panic!("list schema failed: {}", e);
            }
        }
    }
}
