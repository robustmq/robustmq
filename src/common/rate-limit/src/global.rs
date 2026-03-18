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

use common_base::error::ResultCommonError;
use dashmap::DashMap;
use governor::{Quota, RateLimiter};
use std::{num::NonZero, sync::Arc};

use crate::ArcRateLimiter;

#[derive(Clone)]
pub struct GlobalRateLimiterManager {
    http_limits: DashMap<String, ArcRateLimiter>,
    network_connection_limit: ArcRateLimiter,
}
impl Default for GlobalRateLimiterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalRateLimiterManager {
    pub fn new() -> Self {
        let non_zero = NonZero::new(42).unwrap();
        GlobalRateLimiterManager {
            http_limits: DashMap::with_capacity(2),
            network_connection_limit: Arc::new(RateLimiter::direct(Quota::per_second(non_zero))),
        }
    }

    pub async fn wait_http_limit(&self, uri: String) -> ResultCommonError {
        if let Some(limit) = self.http_limits.get(&uri) {
            limit.until_ready().await;
        } else {
            let non_zero = NonZero::new(42).unwrap();
            let limit = Arc::new(RateLimiter::direct(Quota::per_second(non_zero)));
            limit.until_ready().await;
            self.http_limits.insert(uri, limit);
        }
        Ok(())
    }

    pub async fn wait_network_connection_limit(&self) -> ResultCommonError {
        self.network_connection_limit.until_ready().await;
        Ok(())
    }
}
