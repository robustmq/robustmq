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

use crate::cache::BrokerCacheManager;
use common_base::{error::common::CommonError, tools::now_second};
use common_config::broker::broker_config;
use grpc_clients::meta::common::call::{create_tenant, delete_tenant, list_tenant};
use grpc_clients::pool::ClientPool;
use metadata_struct::tenant::Tenant;
use protocol::meta::meta_service_common::{
    CreateTenantRequest, DeleteTenantRequest, ListTenantRequest,
};
use std::sync::Arc;

pub const DEFAULT_TENANT_NAME: &str = "default";
pub const DEFAULT_TENANT_DESC: &str = "Default tenant";

/// 启动时尝试初始化默认租户。
/// 若 default 租户已存在则跳过，否则创建并写入缓存。
pub async fn try_init_default_tenant(
    broker_cache: &Arc<BrokerCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), CommonError> {
    let storage = TenantStorage::new(client_pool.clone());
    let exists = storage
        .list(Some(DEFAULT_TENANT_NAME))
        .await?
        .into_iter()
        .any(|t| t.tenant_name == DEFAULT_TENANT_NAME);

    if exists {
        return Ok(());
    }

    storage
        .create(DEFAULT_TENANT_NAME, DEFAULT_TENANT_DESC)
        .await?;

    broker_cache.add_tenant(Tenant {
        tenant_name: DEFAULT_TENANT_NAME.to_string(),
        desc: DEFAULT_TENANT_DESC.to_string(),
        create_time: now_second(),
    });
    Ok(())
}

pub struct TenantStorage {
    client_pool: Arc<ClientPool>,
}

impl TenantStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TenantStorage { client_pool }
    }

    pub async fn create(&self, tenant_name: &str, desc: &str) -> Result<(), CommonError> {
        let conf = broker_config();
        let request = CreateTenantRequest {
            tenant_name: tenant_name.to_string(),
            desc: desc.to_string(),
        };
        create_tenant(&self.client_pool, &conf.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete(&self, tenant_name: &str) -> Result<(), CommonError> {
        let conf = broker_config();
        let request = DeleteTenantRequest {
            tenant_name: tenant_name.to_string(),
        };
        delete_tenant(&self.client_pool, &conf.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<Tenant>, CommonError> {
        self.list(None).await
    }

    pub async fn list(&self, tenant_name: Option<&str>) -> Result<Vec<Tenant>, CommonError> {
        let conf = broker_config();
        let request = ListTenantRequest {
            tenant_name: tenant_name.unwrap_or_default().to_string(),
        };
        let mut stream =
            list_tenant(&self.client_pool, &conf.get_meta_service_addr(), request).await?;

        let mut tenants = Vec::new();
        while let Some(reply) = stream.message().await? {
            let tenant = Tenant::decode(&reply.tenant)?;
            tenants.push(tenant);
        }
        Ok(tenants)
    }
}
