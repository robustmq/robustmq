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
    placement_add_share_group_member, placement_create_share_group, placement_delete_share_group,
    placement_delete_share_group_member, placement_list_share_group,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::share_group::{ShareGroup, ShareGroupMember, ShareGroupParams};
use protocol::meta::meta_service_common::{
    AddShareGroupMemberRequest, CreateShareGroupRequest, DeleteShareGroupMemberRequest,
    DeleteShareGroupRequest, ListShareGroupRequest,
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
        source: ShareGroupParams,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let params = serde_json::to_vec(&source)?;
        let request = CreateShareGroupRequest {
            tenant: tenant.to_owned(),
            group: group_name.to_owned(),
            params,
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
    ) -> Result<Option<ShareGroup>, CommonError> {
        let config = broker_config();
        let request = ListShareGroupRequest {
            tenant: tenant.to_owned(),
            group: group_name.to_owned(),
        };
        let reply =
            placement_list_share_group(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        if let Some(raw) = reply.groups.first() {
            return Ok(Some(ShareGroup::decode(raw)?));
        }
        Ok(None)
    }

    pub async fn list_all(&self) -> Result<Vec<ShareGroup>, CommonError> {
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
            results.push(ShareGroup::decode(raw)?);
        }
        Ok(results)
    }

    pub async fn list_by_tenant(&self, tenant: &str) -> Result<Vec<ShareGroup>, CommonError> {
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
            results.push(ShareGroup::decode(raw)?);
        }
        Ok(results)
    }

    pub async fn add_member(&self, member: &ShareGroupMember) -> Result<(), CommonError> {
        let config = broker_config();
        let request = AddShareGroupMemberRequest {
            data: member.encode()?,
        };

        placement_add_share_group_member(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_member(
        &self,
        tenant: &str,
        group_name: &str,
        broker_id: u64,
        connect_id: u64,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteShareGroupMemberRequest {
            tenant: tenant.to_string(),
            group_name: group_name.to_owned(),
            broker_id,
            connect_id,
        };
        placement_delete_share_group_member(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }
}
