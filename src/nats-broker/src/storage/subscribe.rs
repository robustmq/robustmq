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
use grpc_clients::meta::nats::call::{
    placement_create_nats_subscribe, placement_delete_nats_subscribe, placement_list_nats_subscribe,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::nats::subscribe::NatsSubscribe;
use protocol::meta::meta_service_nats::ListNatsSubscribeReply;
use protocol::meta::meta_service_nats::{
    CreateNatsSubscribeRequest, DeleteNatsSubscribeRequest, ListNatsSubscribeRequest,
    NatsSubscribeKey,
};
use std::sync::Arc;
use tonic::Streaming;

pub struct NatsSubscribeStorage {
    client_pool: Arc<ClientPool>,
}

impl NatsSubscribeStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        NatsSubscribeStorage { client_pool }
    }

    pub async fn save(
        &self,
        tenant: &str,
        subscribes: Vec<&NatsSubscribe>,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let mut encoded = Vec::with_capacity(subscribes.len());
        for s in subscribes {
            encoded.push(s.encode()?);
        }
        let request = CreateNatsSubscribeRequest {
            tenant: tenant.to_string(),
            subscribes: encoded,
        };
        placement_create_nats_subscribe(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete(
        &self,
        tenant: &str,
        client_id: &str,
        sid: &str,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteNatsSubscribeRequest {
            tenant: tenant.to_string(),
            keys: vec![NatsSubscribeKey {
                client_id: client_id.to_string(),
                sid: sid.to_string(),
            }],
        };
        placement_delete_nats_subscribe(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn list(
        &self,
        tenant: &str,
        client_id: &str,
    ) -> Result<Vec<NatsSubscribe>, CommonError> {
        let config = broker_config();
        let request = ListNatsSubscribeRequest {
            tenant: tenant.to_string(),
            client_id: client_id.to_string(),
        };
        let mut stream: Streaming<ListNatsSubscribeReply> = placement_list_nats_subscribe(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;

        let mut subscribes = Vec::new();
        while let Some(reply) = stream.message().await? {
            subscribes.push(NatsSubscribe::decode(&reply.subscribe)?);
        }
        Ok(subscribes)
    }
}
