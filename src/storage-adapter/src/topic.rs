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

use std::{sync::Arc, time::Duration};

use crate::driver::StorageDriverManager;
use broker_core::cache::BrokerCacheManager;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::{meta::mqtt::call::placement_create_topic, pool::ClientPool};
use metadata_struct::{mqtt::topic::Topic, storage::shard::EngineShardConfig};
use protocol::meta::meta_service_mqtt::CreateTopicRequest;
use tokio::time::{sleep, timeout};

pub async fn create_topic_full(
    broker_cache: &Arc<BrokerCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    topic: &Topic,
    shard_config: &EngineShardConfig,
) -> Result<(), CommonError> {
    let conf = broker_config();
    let request = CreateTopicRequest {
        topic_name: topic.topic_name.clone(),
        content: topic.encode()?,
    };
    placement_create_topic(client_pool, &conf.get_meta_service_addr(), request).await?;

    // wait topic create complete with timeout (30 seconds)
    let wait_result = timeout(Duration::from_secs(30), async {
        loop {
            if broker_cache.get_topic_by_name(&topic.topic_name).is_some() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    if wait_result.is_err() {
        return Err(CommonError::CommonError(format!(
            "Timeout waiting for topic '{}' to be created after 30 seconds",
            topic.topic_name
        )));
    }

    // todo create topic message storage
    storage_driver_manager
        .create_storage_resource(&topic.topic_name, shard_config)
        .await?;
    Ok(())
}
