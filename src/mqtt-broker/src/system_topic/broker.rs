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

use broker_core::cluster::ClusterStorage;
use common_base::tools::now_second;
use common_base::version::version;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

use super::{
    replace_topic_name, report_system_data, write_topic_data, SYSTEM_TOPIC_BROKERS,
    SYSTEM_TOPIC_BROKERS_DATETIME, SYSTEM_TOPIC_BROKERS_SYSDESCR, SYSTEM_TOPIC_BROKERS_UPTIME,
    SYSTEM_TOPIC_BROKERS_VERSION,
};
use crate::handler::cache::MQTTCacheManager;

pub(crate) async fn report_cluster_status(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS.to_string());
    if let Some(record) = build_node_cluster(&topic_name, client_pool).await {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_pool,
            topic_name,
            record,
        )
        .await;
    }
}

pub(crate) async fn report_broker_version(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_VERSION,
        || async { version() },
    )
    .await;
}

pub(crate) async fn report_broker_time(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    //  report system uptime
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_UPTIME,
        || async {
            let start_long_time: u64 = now_second() - metadata_cache.broker_cache.get_start_time();
            start_long_time.to_string()
        },
    )
    .await;

    // report system datetime
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_DATETIME,
        || async { now_second().to_string() },
    )
    .await;
}

pub(crate) async fn report_broker_sysdescr(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
) {
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_SYSDESCR,
        || async { format!("report broker sysdescr error,error:{}", os_info::get()) },
    )
    .await;
}

async fn build_node_cluster(topic_name: &str, client_pool: &Arc<ClientPool>) -> Option<Record> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let node_list = match cluster_storage.node_list().await {
        Ok(data) => data,
        Err(e) => {
            error!("build node cluster {}", e.to_string());
            return None;
        }
    };

    let content = match serde_json::to_string(&node_list) {
        Ok(content) => content,
        Err(e) => {
            error!(
                "Failed to serialize node-list, failure message :{}",
                e.to_string()
            );
            return None;
        }
    };

    MqttMessage::build_system_topic_message(topic_name.to_string(), content)
}

#[cfg(test)]
mod tests {
    use std::env;

    #[tokio::test]
    async fn os_info_test() {
        let info = os_info::get();
        println!("{info}");
    }

    #[tokio::test]
    async fn version_test() {
        let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "-".to_string());
        println!("{version}");
        assert_ne!(version, "-".to_string());
    }
}
