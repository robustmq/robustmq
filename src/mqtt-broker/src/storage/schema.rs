use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::{placement::inner::call::list_schema, pool::ClientPool};
use metadata_struct::schema::SchemaData;
use protocol::placement_center::placement_center_inner::{ListSchemaReply, ListSchemaRequest};
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
