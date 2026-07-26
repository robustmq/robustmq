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

use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, info};

pub(crate) const IDLE_SLEEP_MS: u64 = 100;
pub(crate) const LOW_LOAD_SLEEP_MS: u64 = 20;
pub(crate) const LOW_LOAD_THRESHOLD: usize = 10;

pub(crate) fn should_stop(val: Result<bool, broadcast::error::RecvError>, label: &str) -> bool {
    match val {
        Ok(true) => {
            info!("{} stopped", label);
            true
        }
        Ok(false) => false,
        Err(broadcast::error::RecvError::Closed) => {
            info!("{} stop channel closed", label);
            true
        }
        Err(broadcast::error::RecvError::Lagged(n)) => {
            debug!("{} stop channel lagged, skipped {}", label, n);
            false
        }
    }
}

pub(crate) async fn adaptive_sleep(count: usize) {
    if count == 0 {
        sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
    } else if count < LOW_LOAD_THRESHOLD {
        sleep(Duration::from_millis(LOW_LOAD_SLEEP_MS)).await;
    }
}
