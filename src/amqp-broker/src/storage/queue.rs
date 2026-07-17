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
    placement_delete_queue, placement_list_queue, placement_set_queue,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::amqp::queue::AmqpQueue;
use protocol::meta::meta_service_amqp::{DeleteQueueRequest, ListQueueRequest, SetQueueRequest};

pub struct QueueStorage {
    client_pool: Arc<ClientPool>,
}

impl QueueStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        QueueStorage { client_pool }
    }

    pub async fn set_queue(&self, queue: &AmqpQueue) -> Result<(), CommonError> {
        let config = broker_config();
        let request = SetQueueRequest {
            queue: queue.encode()?,
        };
        placement_set_queue(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_queue(&self, tenant: &str, queue_name: &str) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteQueueRequest {
            tenant: tenant.to_string(),
            queue_name: queue_name.to_string(),
        };
        placement_delete_queue(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn list_queue_by_tenant(&self, tenant: &str) -> Result<Vec<AmqpQueue>, CommonError> {
        let config = broker_config();
        let request = ListQueueRequest {
            tenant: tenant.to_string(),
        };
        let reply =
            placement_list_queue(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let mut results = Vec::with_capacity(reply.queues.len());
        for raw in reply.queues {
            results.push(AmqpQueue::decode(&raw)?);
        }
        Ok(results)
    }
}
