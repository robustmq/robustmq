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

use std::{num::NonZero, sync::Arc};

use common_base::error::ResultCommonError;
use dashmap::DashMap;
use governor::{Quota, RateLimiter};

type ArcRateLimiter = Arc<
    RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::QuantaClock,
        governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>,
    >,
>;
pub struct RateLimiterManager {
    http_limits: DashMap<String, ArcRateLimiter>,
    grpc_limits: DashMap<String, ArcRateLimiter>,
    mqtt_publish: DashMap<String, ArcRateLimiter>,
    mqtt_connection: ArcRateLimiter,
}
impl Default for RateLimiterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiterManager {
    pub fn new() -> Self {
        let non_zero = NonZero::new(42).unwrap();
        let mqtt_connection = Arc::new(RateLimiter::direct(Quota::per_second(non_zero)));
        RateLimiterManager {
            http_limits: DashMap::with_capacity(2),
            grpc_limits: DashMap::with_capacity(2),
            mqtt_publish: DashMap::with_capacity(2),
            mqtt_connection,
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

    pub async fn wait_grpc_limit(&self, method: String) -> ResultCommonError {
        if let Some(limit) = self.grpc_limits.get(&method) {
            limit.until_ready().await;
        } else {
            let non_zero = NonZero::new(42).unwrap();
            let limit = Arc::new(RateLimiter::direct(Quota::per_second(non_zero)));
            limit.until_ready().await;
            self.http_limits.insert(method, limit);
        }
        Ok(())
    }

    pub async fn wait_mqtt_publish_limit(&self, client_id: String) -> ResultCommonError {
        if let Some(limit) = self.mqtt_publish.get(&client_id) {
            limit.until_ready().await;
        } else {
            let non_zero = NonZero::new(42).unwrap();
            let limit = Arc::new(RateLimiter::direct(Quota::per_second(non_zero)));
            limit.until_ready().await;
            self.http_limits.insert(client_id, limit);
        }
        Ok(())
    }

    pub async fn wait_mqtt_connection_limit(&self) -> ResultCommonError {
        self.mqtt_connection.until_ready().await;
        Ok(())
    }
}
