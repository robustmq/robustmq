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
use common_base::node_status::NodeStatus;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::warn;

const WAIT_CLUSTER_WARN_THRESHOLD: Duration = Duration::from_secs(1);
const WAIT_CLUSTER_MAX_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn wait_cluster_running(cache_manager: &Arc<BrokerCacheManager>) -> Result<(), String> {
    let start = Instant::now();
    let mut warned = false;

    loop {
        if cache_manager.get_status().await == NodeStatus::Running {
            if warned {
                warn!(
                    "Cluster became running after waiting {:.2}s",
                    start.elapsed().as_secs_f64()
                );
            }
            return Ok(());
        }

        let elapsed = start.elapsed();
        if elapsed >= WAIT_CLUSTER_MAX_TIMEOUT {
            warn!(
                "wait_cluster_running timed out after {}s, cluster status is not Running",
                WAIT_CLUSTER_MAX_TIMEOUT.as_secs()
            );
            return Err(format!(
                "Cluster not ready after waiting {}s, current status is not Running",
                WAIT_CLUSTER_MAX_TIMEOUT.as_secs()
            ));
        }

        if elapsed >= WAIT_CLUSTER_WARN_THRESHOLD {
            warned = true;
            warn!(
                "Waiting for cluster to be running... elapsed={:.2}s, max_timeout={}s",
                elapsed.as_secs_f64(),
                WAIT_CLUSTER_MAX_TIMEOUT.as_secs()
            );
        }

        sleep(Duration::from_secs(1)).await;
    }
}
