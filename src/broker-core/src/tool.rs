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
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub async fn wait_cluster_running(cache_manager: &Arc<BrokerCacheManager>) {
    loop {
        if let Some(status) = cache_manager.status.get(&cache_manager.cluster_name) {
            if status.clone() == NodeStatus::Running {
                break;
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}
