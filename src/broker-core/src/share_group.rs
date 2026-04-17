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
use grpc_clients::meta::common::call::{
    placement_create_share_group, placement_delete_share_group, placement_list_share_group,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::share_group::ShareGroupLeader;
use protocol::meta::meta_service_common::{
    CreateShareGroupRequest, DeleteShareGroupRequest, ListShareGroupRequest,
};
use std::sync::Arc;

pub struct ShareGroupStorage {
    client_pool: Arc<ClientPool>,
}

impl ShareGroupStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        ShareGroupStorage { client_pool }
    }

    pub async fn create(
        &self,
        tenant: &str,
        group_name: &str,
        source: i32,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = CreateShareGroupRequest {
            tenant: tenant.to_owned(),
            group: group_name.to_owned(),
            source,
        };
        placement_create_share_group(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, tenant: &str, group_name: &str) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteShareGroupRequest {
            tenant: tenant.to_owned(),
            group: group_name.to_owned(),
        };
        placement_delete_share_group(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn get(
        &self,
        tenant: &str,
        group_name: &str,
    ) -> Result<Option<ShareGroupLeader>, CommonError> {
        let config = broker_config();
        let request = ListShareGroupRequest {
            tenant: tenant.to_owned(),
            group: group_name.to_owned(),
        };
        let reply =
            placement_list_share_group(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        if let Some(raw) = reply.groups.first() {
            return Ok(Some(ShareGroupLeader::decode(raw)?));
        }
        Ok(None)
    }

    pub async fn list_all(&self) -> Result<Vec<ShareGroupLeader>, CommonError> {
        let config = broker_config();
        let request = ListShareGroupRequest {
            tenant: String::new(),
            group: String::new(),
        };
        let reply =
            placement_list_share_group(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let mut results = Vec::with_capacity(reply.groups.len());
        for raw in reply.groups.iter() {
            results.push(ShareGroupLeader::decode(raw)?);
        }
        Ok(results)
    }

    pub async fn list_by_tenant(&self, tenant: &str) -> Result<Vec<ShareGroupLeader>, CommonError> {
        let config = broker_config();
        let request = ListShareGroupRequest {
            tenant: tenant.to_owned(),
            group: String::new(),
        };
        let reply =
            placement_list_share_group(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let mut results = Vec::with_capacity(reply.groups.len());
        for raw in reply.groups.iter() {
            results.push(ShareGroupLeader::decode(raw)?);
        }
        Ok(results)
    }
}
