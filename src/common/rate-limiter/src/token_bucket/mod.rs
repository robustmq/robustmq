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

use axum::async_trait;
use governor::middleware::NoOpMiddleware;
use governor::{clock, Quota, RateLimiter};
use nonzero_ext::nonzero;
use std::sync::Arc;
use std::time::Duration;

static RATE_LIMITER_MANAGER: std::sync::LazyLock<Arc<dyn RateLimiterManagerExt + Send + Sync>> =
    std::sync::LazyLock::new(|| Arc::new(CustomRateLimiterManager::new()));

type DefaultRateLimiter = Arc<
    RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        clock::QuantaUpkeepClock,
        NoOpMiddleware<clock::QuantaInstant>,
    >,
>;

pub fn get_default_rate_limiter_manager() -> Arc<dyn RateLimiterManagerExt + Send + Sync> {
    RATE_LIMITER_MANAGER.clone()
}

#[async_trait]
pub trait RateLimiterManagerExt: Send + Sync {
    async fn get_or_register(&self, name: String) -> Arc<dyn RateLimiterExt + Send + Sync>;
    async fn delete(&self, name: String);
}

#[async_trait]
pub trait RateLimiterExt: Send + Sync {
    async fn check_key(&self) -> bool;
}

pub struct CustomRateLimiterManager {
    clock: clock::QuantaUpkeepClock,
    store: dashmap::DashMap<String, Arc<dyn RateLimiterExt + Send + Sync>>,
}

impl CustomRateLimiterManager {
    pub fn new() -> CustomRateLimiterManager {
        CustomRateLimiterManager {
            clock: clock::QuantaUpkeepClock::from_interval(Duration::from_micros(10))
                .expect("Could not spawn upkeep thread"),
            store: dashmap::DashMap::new(),
        }
    }
}

#[async_trait]
impl RateLimiterManagerExt for CustomRateLimiterManager {
    async fn get_or_register(&self, name: String) -> Arc<dyn RateLimiterExt + Send + Sync> {
        self.store
            .entry(name)
            .or_insert(Arc::new(CustomRateLimiter {
                limiter: Arc::new(RateLimiter::direct_with_clock(
                    Quota::per_second(nonzero!(50u32)),
                    self.clock.clone(),
                )),
            }))
            .clone()
    }
    async fn delete(&self, name: String) {
        self.store.remove(&name);
    }
}

pub struct CustomRateLimiter {
    limiter: DefaultRateLimiter,
}

#[async_trait]
impl RateLimiterExt for CustomRateLimiter {
    async fn check_key(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::AtomicI32;

    use super::*;

    #[tokio::test]
    async fn token_bucket_test() {
        let limiter_manager = Arc::new(CustomRateLimiterManager::new());
        let mut handles = Vec::new();
        let okey = Arc::new(AtomicI32::new(0));
        let fail = Arc::new(AtomicI32::new(0));

        for _ in 0..100 {
            let lm = limiter_manager.clone();
            let okey = okey.clone();
            let fail = fail.clone();

            let handle = tokio::spawn(async move {
                if lm
                    .get_or_register("test_limiter".to_string())
                    .await
                    .check_key()
                    .await
                {
                    okey.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                } else {
                    fail.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        let success_count = okey.load(std::sync::atomic::Ordering::SeqCst);
        let fail_count = fail.load(std::sync::atomic::Ordering::SeqCst);

        println!("Success: {}, Fail: {}", success_count, fail_count);

        assert!(success_count >= 48 && success_count <= 52);
        assert!(fail_count >= 48 && fail_count <= 52);
    }
}
