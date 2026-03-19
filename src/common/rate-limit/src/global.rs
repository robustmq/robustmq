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

use crate::{ArcLockRateLimiter, ArcRateLimiter};
use common_base::error::{common::CommonError, ResultCommonError};
use common_config::broker::broker_config;
use dashmap::DashMap;
use governor::{Quota, RateLimiter};
use std::{num::NonZero, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct GlobalRateLimiterManager {
    // (uri, ArcRateLimiter)
    http_limits: DashMap<String, ArcRateLimiter>,

    // ArcRateLimiter
    network_connection_rate: ArcLockRateLimiter,
}

impl GlobalRateLimiterManager {
    pub fn new(connection_rate: u32) -> Result<Self, Box<CommonError>> {
        let connection_non_zero = NonZero::new(connection_rate).ok_or_else(|| {
            Box::new(CommonError::InvalidRateLimitValue(
                "connection_rate".to_string(),
            ))
        })?;

        Ok(GlobalRateLimiterManager {
            http_limits: DashMap::with_capacity(8),
            network_connection_rate: Arc::new(RwLock::new(RateLimiter::direct(Quota::per_second(
                connection_non_zero,
            )))),
        })
    }

    pub fn set_http_rate(&self, uri: &str, rate: u32) -> Result<ArcRateLimiter, Box<CommonError>> {
        let non_zero = NonZero::new(rate)
            .ok_or_else(|| Box::new(CommonError::InvalidRateLimitValue("http_rate".to_string())))?;
        let limit = Arc::new(RateLimiter::direct(Quota::per_second(non_zero)));
        self.http_limits.insert(uri.to_string(), limit.clone());
        Ok(limit)
    }

    pub async fn set_network_connection_rate(&self, rate: u32) -> ResultCommonError {
        let mut write = self.network_connection_rate.write().await;
        let non_zero = NonZero::new(rate).ok_or_else(|| {
            CommonError::InvalidRateLimitValue("network_connection_rate".to_string())
        })?;
        *write = RateLimiter::direct(Quota::per_second(non_zero));
        Ok(())
    }

    pub async fn http_uri_rate_limit(&self, uri: String) -> Result<(), Box<CommonError>> {
        if let Some(limit) = self.http_limits.get(&uri) {
            limit.until_ready().await;
        } else {
            let config = broker_config();
            let connection_non_zero = NonZero::new(
                config.cluster_limit.max_network_connection_rate,
            )
            .ok_or_else(|| {
                Box::new(CommonError::InvalidRateLimitValue(
                    "network_connection_rate".to_string(),
                ))
            })?;
            let limit = Arc::new(RateLimiter::direct(Quota::per_second(connection_non_zero)));
            limit.until_ready().await;
            self.http_limits.insert(uri, limit);
        }
        Ok(())
    }

    pub async fn network_connection_rate_limit(&self) -> ResultCommonError {
        let read = self.network_connection_rate.read().await;
        read.until_ready().await;
        Ok(())
    }
}
