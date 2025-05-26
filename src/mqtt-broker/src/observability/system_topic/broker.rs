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

use std::env;
use std::sync::Arc;

use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use storage_adapter::storage::StorageAdapter;
use tracing::error;

use super::{
    replace_topic_name, report_system_data, write_topic_data, SYSTEM_TOPIC_BROKERS,
    SYSTEM_TOPIC_BROKERS_DATETIME, SYSTEM_TOPIC_BROKERS_SYSDESCR, SYSTEM_TOPIC_BROKERS_UPTIME,
    SYSTEM_TOPIC_BROKERS_VERSION,
};
use crate::handler::cache::CacheManager;
use crate::storage::cluster::ClusterStorage;
use crate::BROKER_START_TIME;

pub(crate) async fn report_cluster_status<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
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

pub(crate) async fn report_broker_version<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_VERSION,
        || async { env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "-".to_string()) },
    )
    .await;
}

pub(crate) async fn report_broker_time<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    //  report system uptime
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_UPTIME,
        || async {
            let start_long_time: u64 = now_second() - *BROKER_START_TIME;
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

pub(crate) async fn report_broker_sysdescr<S>(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_system_data(
        client_pool,
        metadata_cache,
        message_storage_adapter,
        SYSTEM_TOPIC_BROKERS_SYSDESCR,
        || async { format!("{}", os_info::get()) },
    )
    .await;
}

async fn build_node_cluster(topic_name: &str, client_pool: &Arc<ClientPool>) -> Option<Record> {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let node_list = match cluster_storage.node_list().await {
        Ok(data) => data,
        Err(e) => {
            error!("{}", e.to_string());
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
        println!("{}", info);
    }

    #[tokio::test]
    async fn version_test() {
        let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "-".to_string());
        println!("{}", version);
        assert_ne!(version, "-".to_string());
    }
}
