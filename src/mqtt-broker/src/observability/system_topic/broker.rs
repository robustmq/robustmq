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

use clients::poll::ClientPool;
use common_base::tools::now_second;
use log::error;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use storage_adapter::storage::StorageAdapter;

use super::{
    replace_topic_name, write_topic_data, SYSTEM_TOPIC_BROKERS, SYSTEM_TOPIC_BROKERS_DATETIME,
    SYSTEM_TOPIC_BROKERS_SYSDESCR, SYSTEM_TOPIC_BROKERS_UPTIME, SYSTEM_TOPIC_BROKERS_VERSION,
};
use crate::handler::cache::CacheManager;
use crate::storage::cluster::ClusterStorage;
use crate::BROKER_START_TIME;

pub(crate) async fn report_broker_info<S>(
    client_poll: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    report_cluster_status(client_poll, metadata_cache, message_storage_adapter).await;
    report_broker_version(client_poll, metadata_cache, message_storage_adapter).await;
    report_broker_time(client_poll, metadata_cache, message_storage_adapter).await;
    report_broker_sysdescr(client_poll, metadata_cache, message_storage_adapter).await;
}

async fn report_cluster_status<S>(
    client_poll: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS.to_string());
    if let Some(record) = build_node_cluster(&topic_name, client_poll).await {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_poll,
            topic_name,
            record,
        )
        .await;
    }
}

async fn report_broker_version<S>(
    client_poll: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_VERSION.to_string());
    let version = match env::var("CARGO_PKG_VERSION") {
        Ok(data) => data,
        Err(_) => "-".to_string(),
    };

    if let Some(record) = MqttMessage::build_system_topic_message(topic_name.clone(), version) {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_poll,
            topic_name,
            record,
        )
        .await;
    }
}

async fn report_broker_time<S>(
    client_poll: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_UPTIME.to_string());
    let start_long_time: u64 = now_second() - *BROKER_START_TIME;
    if let Some(record) =
        MqttMessage::build_system_topic_message(topic_name.clone(), start_long_time.to_string())
    {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_poll,
            topic_name,
            record,
        )
        .await;
    }

    let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_DATETIME.to_string());
    if let Some(record) =
        MqttMessage::build_system_topic_message(topic_name.clone(), now_second().to_string())
    {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_poll,
            topic_name,
            record,
        )
        .await;
    }
}

async fn report_broker_sysdescr<S>(
    client_poll: &Arc<ClientPool>,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_SYSDESCR.to_string());
    let info = format!("{}", os_info::get());
    if let Some(record) = MqttMessage::build_system_topic_message(topic_name.clone(), info) {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_poll,
            topic_name,
            record,
        )
        .await;
    }
}

async fn build_node_cluster(topic_name: &str, client_poll: &Arc<ClientPool>) -> Option<Record> {
    let cluster_storage = ClusterStorage::new(client_poll.clone());
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
        let version = match env::var("CARGO_PKG_VERSION") {
            Ok(data) => data,
            Err(_) => "-".to_string(),
        };
        println!("{}", version);
        assert_ne!(version, "-".to_string());
    }
}
