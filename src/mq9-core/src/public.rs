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

use std::sync::Arc;

use broker_core::cache::NodeCacheManager;
use common_base::error::common::CommonError;
use common_config::storage::StorageType;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    storage::shard::EngineShardConfig,
    tenant::DEFAULT_TENANT,
    topic::{Topic, TopicSource},
};
use serde::{Deserialize, Serialize};
use storage_adapter::{driver::StorageDriverManager, topic::create_topic_full};

pub const MQ9_SYSTEM_PUBLIC_MAIL: &str = "$SYSTEM.PUBLIC";

/// Returns true if the given mail_address is a reserved system mailbox.
pub fn is_system_mailbox(mail_address: &str) -> bool {
    mail_address == MQ9_SYSTEM_PUBLIC_MAIL
}

pub async fn try_init_system_email(
    node_cache: &Arc<NodeCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), CommonError> {
    let tenant = DEFAULT_TENANT;
    if let Some(topic) = node_cache.get_topic_by_name(tenant, MQ9_SYSTEM_PUBLIC_MAIL) {
        // Topic exists in metadata; ensure the storage shard is also provisioned.
        let shard_config = EngineShardConfig {
            replica_num: topic.replication,
            storage_type: topic.storage_type,
            max_segment_size: topic.config.max_segment_size,
            max_record_num: topic.config.max_record_num,
            retention_sec: topic.config.retention_sec,
        };
        storage_driver_manager
            .create_storage_resource(tenant, MQ9_SYSTEM_PUBLIC_MAIL, &shard_config)
            .await?;
        return Ok(());
    }

    let topic = Topic::new(tenant, MQ9_SYSTEM_PUBLIC_MAIL, StorageType::EngineRocksDB)
        .with_source(TopicSource::MQ9);

    create_topic_full(node_cache, storage_driver_manager, client_pool, &topic).await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct StoragePublicData {
    pub mail_address: String,
    pub desc: String,
    pub ttl: u64,
    pub create_at: u64,
}
