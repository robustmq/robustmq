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
    placement_create_nats_subject, placement_delete_nats_subject, placement_list_nats_subject,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::nats::subject::NatsSubject;
use protocol::meta::meta_service_nats::ListNatsSubjectReply;
use protocol::meta::meta_service_nats::{
    CreateNatsSubjectRequest, DeleteNatsSubjectRequest, ListNatsSubjectRequest,
};
use std::sync::Arc;
use tonic::Streaming;

pub struct NatsSubjectStorage {
    client_pool: Arc<ClientPool>,
}

impl NatsSubjectStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        NatsSubjectStorage { client_pool }
    }

    pub async fn save(&self, subject: &NatsSubject) -> Result<(), CommonError> {
        let config = broker_config();
        let request = CreateNatsSubjectRequest {
            tenant: subject.tenant.clone(),
            subject: subject.encode()?,
        };
        placement_create_nats_subject(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, tenant: &str, name: &str) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteNatsSubjectRequest {
            tenant: tenant.to_string(),
            name: name.to_string(),
        };
        placement_delete_nats_subject(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn list(&self, tenant: &str) -> Result<Vec<NatsSubject>, CommonError> {
        let config = broker_config();
        let request = ListNatsSubjectRequest {
            tenant: tenant.to_string(),
            name: String::new(),
        };
        let mut stream: Streaming<ListNatsSubjectReply> = placement_list_nats_subject(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;

        let mut subjects = Vec::new();
        while let Some(reply) = stream.message().await? {
            subjects.push(NatsSubject::decode(&reply.subject)?);
        }
        Ok(subjects)
    }
}
