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

use std::sync::Arc;

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::meta::amqp::call::{
    placement_delete_binding, placement_list_binding, placement_set_binding,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::amqp::binding::{AmqpBinding, AmqpBindingDestinationType};
use protocol::meta::meta_service_amqp::{
    DeleteBindingRequest, ListBindingRequest, SetBindingRequest,
};

pub struct BindingStorage {
    client_pool: Arc<ClientPool>,
}

impl BindingStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        BindingStorage { client_pool }
    }

    pub async fn set_binding(&self, binding: &AmqpBinding) -> Result<(), CommonError> {
        let config = broker_config();
        let request = SetBindingRequest {
            binding: binding.encode()?,
        };
        placement_set_binding(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_binding(
        &self,
        tenant: &str,
        source: &str,
        destination: &str,
        destination_type: &AmqpBindingDestinationType,
        routing_key: &str,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteBindingRequest {
            tenant: tenant.to_string(),
            source: source.to_string(),
            destination: destination.to_string(),
            destination_type: destination_type.as_str().to_string(),
            routing_key: routing_key.to_string(),
        };
        placement_delete_binding(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn list_binding_by_tenant(
        &self,
        tenant: &str,
    ) -> Result<Vec<AmqpBinding>, CommonError> {
        let config = broker_config();
        let request = ListBindingRequest {
            tenant: tenant.to_string(),
        };
        let reply =
            placement_list_binding(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let mut results = Vec::with_capacity(reply.bindings.len());
        for raw in reply.bindings {
            results.push(AmqpBinding::decode(&raw)?);
        }
        Ok(results)
    }
}
