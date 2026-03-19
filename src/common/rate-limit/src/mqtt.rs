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

use broker_core::cache::NodeCacheManager;
use common_base::error::{common::CommonError, ResultCommonError};
use dashmap::DashMap;
use governor::{Quota, RateLimiter};
use std::{num::NonZero, sync::Arc};
use tokio::sync::RwLock;

use crate::{ArcLockRateLimiter, ArcRateLimiter};

#[derive(Clone)]
pub struct MQTTRateLimiterManager {
    pub node_cache: Arc<NodeCacheManager>,
    // publish
    node_publish_message_rate: ArcLockRateLimiter,
    tenant_publish_message_rate: DashMap<String, ArcRateLimiter>,

    // create connection
    node_create_connection_rate: ArcLockRateLimiter,
    tenant_create_connection_rate: DashMap<String, ArcRateLimiter>,
}

impl MQTTRateLimiterManager {
    pub fn new(
        node_cache: Arc<NodeCacheManager>,
        publish_rate: u32,
        create_connection_rate: u32,
    ) -> Result<Self, Box<CommonError>> {
        let publish_non_zero = NonZero::new(publish_rate).ok_or_else(|| {
            Box::new(CommonError::InvalidRateLimitValue(
                "publish_rate".to_string(),
            ))
        })?;

        let create_connection_non_zero = NonZero::new(create_connection_rate).ok_or_else(|| {
            Box::new(CommonError::InvalidRateLimitValue(
                "create_connection_rate".to_string(),
            ))
        })?;

        Ok(MQTTRateLimiterManager {
            node_cache,
            tenant_publish_message_rate: DashMap::with_capacity(2),
            tenant_create_connection_rate: DashMap::with_capacity(2),
            node_publish_message_rate: Arc::new(RwLock::new(RateLimiter::direct(
                Quota::per_second(publish_non_zero),
            ))),
            node_create_connection_rate: Arc::new(RwLock::new(RateLimiter::direct(
                Quota::per_second(create_connection_non_zero),
            ))),
        })
    }

    pub fn set_tenant_publish_message_rate(
        &self,
        tenant: &str,
        rate: u32,
    ) -> Result<ArcRateLimiter, Box<CommonError>> {
        let non_zero = NonZero::new(rate).ok_or_else(|| {
            Box::new(CommonError::InvalidRateLimitValue(
                "tenant_publish_rate".to_string(),
            ))
        })?;
        let limit = Arc::new(RateLimiter::direct(Quota::per_second(non_zero)));
        self.tenant_publish_message_rate
            .insert(tenant.to_string(), limit.clone());

        Ok(limit.clone())
    }

    pub fn set_tenant_create_connection_rate(
        &self,
        tenant: &str,
        rate: u32,
    ) -> Result<ArcRateLimiter, Box<CommonError>> {
        let non_zero = NonZero::new(rate).ok_or_else(|| {
            Box::new(CommonError::InvalidRateLimitValue(
                "tenant_create_connection_rate".to_string(),
            ))
        })?;
        let limit = Arc::new(RateLimiter::direct(Quota::per_second(non_zero)));
        self.tenant_publish_message_rate
            .insert(tenant.to_string(), limit.clone());

        Ok(limit)
    }

    pub async fn set_node_publish_message_rate(&self, rate: u32) -> ResultCommonError {
        let mut write = self.node_publish_message_rate.write().await;
        let non_zero = NonZero::new(rate)
            .ok_or_else(|| CommonError::InvalidRateLimitValue("node_publish_rate".to_string()))?;
        *write = RateLimiter::direct(Quota::per_second(non_zero));
        Ok(())
    }

    pub async fn set_node_create_connection_rate(&self, rate: u32) -> ResultCommonError {
        let mut write = self.node_create_connection_rate.write().await;
        let non_zero = NonZero::new(rate).ok_or_else(|| {
            CommonError::InvalidRateLimitValue("node_create_connection_rate".to_string())
        })?;
        *write = RateLimiter::direct(Quota::per_second(non_zero));
        Ok(())
    }

    pub async fn connection_rate_limit(&self, tenant: &str) -> ResultCommonError {
        // node
        let limit = self.node_create_connection_rate.read().await;
        limit.until_ready().await;

        // tenant
        if let Some(tenant_limit) = self.tenant_create_connection_rate.get(tenant) {
            tenant_limit.until_ready().await;
        } else if let Some(ten) = self.node_cache.get_tenant(tenant) {
            let limit = self
                .set_tenant_create_connection_rate(tenant, ten.config.max_publish_rate)
                .map_err(|e| *e)?;
            limit.until_ready().await;
        }

        Ok(())
    }

    pub async fn publish_message_rate_limit(&self, tenant: &str) -> ResultCommonError {
        // node
        let limit = self.node_publish_message_rate.read().await;
        limit.until_ready().await;

        // tenant
        if let Some(tenant_limit) = self.tenant_publish_message_rate.get(tenant) {
            tenant_limit.until_ready().await;
        } else if let Some(ten) = self.node_cache.get_tenant(tenant) {
            let limit = self
                .set_tenant_publish_message_rate(tenant, ten.config.max_publish_rate)
                .map_err(|e| *e)?;
            limit.until_ready().await;
        }

        Ok(())
    }
}
