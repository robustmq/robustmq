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
    placement_delete_exchange, placement_list_exchange, placement_set_exchange,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::amqp::exchange::AmqpExchange;
use protocol::meta::meta_service_amqp::{
    DeleteExchangeRequest, ListExchangeRequest, SetExchangeRequest,
};

pub struct ExchangeStorage {
    client_pool: Arc<ClientPool>,
}

impl ExchangeStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        ExchangeStorage { client_pool }
    }

    pub async fn set_exchange(&self, exchange: &AmqpExchange) -> Result<(), CommonError> {
        let config = broker_config();
        let request = SetExchangeRequest {
            exchange: exchange.encode()?,
        };
        placement_set_exchange(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_exchange(
        &self,
        tenant: &str,
        exchange_name: &str,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteExchangeRequest {
            tenant: tenant.to_string(),
            exchange_name: exchange_name.to_string(),
        };
        placement_delete_exchange(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn list_exchange_by_tenant(
        &self,
        tenant: &str,
    ) -> Result<Vec<AmqpExchange>, CommonError> {
        let config = broker_config();
        let request = ListExchangeRequest {
            tenant: tenant.to_string(),
        };
        let reply =
            placement_list_exchange(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let mut results = Vec::with_capacity(reply.exchanges.len());
        for raw in reply.exchanges {
            results.push(AmqpExchange::decode(&raw)?);
        }
        Ok(results)
    }
}
