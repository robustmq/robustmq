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
use common_base::error::ResultCommonError;
use common_config::broker::broker_config;
use grpc_clients::meta::mqtt::call::{create_blacklist, delete_blacklist, list_blacklist};
use grpc_clients::pool::ClientPool;
use metadata_struct::auth::blacklist::SecurityBlackList;
use protocol::meta::meta_service_mqtt::{
    CreateBlacklistRequest, DeleteBlacklistRequest, ListBlacklistRequest,
};
use std::sync::Arc;

pub struct BlackListStorage {
    client_pool: Arc<ClientPool>,
}

impl BlackListStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        BlackListStorage { client_pool }
    }

    pub async fn list_blacklist(&self) -> Result<Vec<SecurityBlackList>, CommonError> {
        let config = broker_config();
        let request = ListBlacklistRequest {
            ..Default::default()
        };
        let reply =
            list_blacklist(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        let mut list = Vec::new();
        for raw in reply.blacklists {
            list.push(SecurityBlackList::decode(&raw)?);
        }
        Ok(list)
    }

    pub async fn save_blacklist(&self, blacklist: SecurityBlackList) -> ResultCommonError {
        let config = broker_config();
        let request = CreateBlacklistRequest {
            blacklist: blacklist.encode()?,
        };
        create_blacklist(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_blacklist(&self, tenant: &str, name: &str) -> ResultCommonError {
        let config = broker_config();
        let request = DeleteBlacklistRequest {
            tenant: tenant.to_string(),
            name: name.to_string(),
        };
        delete_blacklist(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }
}
