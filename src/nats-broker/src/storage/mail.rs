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
use grpc_clients::meta::mq9::call::{
    placement_create_mq9_email, placement_delete_mq9_email, placement_list_mq9_email,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mq9::email::MQ9Email;
use protocol::meta::meta_service_mq9::{CreateEmailRequest, DeleteEmailRequest, ListEmailRequest};
use std::sync::Arc;
use tonic::Streaming;

pub struct Mq9EmailStorage {
    client_pool: Arc<ClientPool>,
}

impl Mq9EmailStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        Mq9EmailStorage { client_pool }
    }

    pub async fn create(&self, email: &MQ9Email) -> Result<(), CommonError> {
        let config = broker_config();
        let request = CreateEmailRequest {
            tenant: email.tenant.clone(),
            content: email.encode()?,
        };
        placement_create_mq9_email(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, tenant: &str, mail_address: &str) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteEmailRequest {
            tenant: tenant.to_string(),
            mail_address: mail_address.to_string(),
        };
        placement_delete_mq9_email(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn list(&self, tenant: &str) -> Result<Vec<MQ9Email>, CommonError> {
        let config = broker_config();
        let request = ListEmailRequest {
            tenant: tenant.to_string(),
        };
        let mut stream: Streaming<_> =
            placement_list_mq9_email(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;

        let mut emails = Vec::new();
        while let Some(reply) = stream.message().await? {
            emails.push(MQ9Email::decode(&reply.email)?);
        }
        Ok(emails)
    }
}
