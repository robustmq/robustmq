use crate::handler::cache::CacheManager;

use super::{write_topic_data, SYSTEM_TOPIC_BROKERS};
use clients::{placement::placement::call::node_list, poll::ClientPool};
use common_base::config::broker_mqtt::broker_mqtt_conf;
use log::error;
use metadata_struct::{
    adapter::record::Record, mqtt::message::MQTTMessage, placement::broker_node::BrokerNode,
};
use protocol::placement_center::generate::placement::NodeListRequest;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

pub async fn report_cluster_status<S>(
    client_poll: Arc<ClientPool>,
    metadata_cache: Arc<CacheManager>,
    message_storage_adapter: Arc<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    if let Some(record) = build_node_cluster(client_poll.clone()).await {
        let topic_name = SYSTEM_TOPIC_BROKERS.to_string();
        if let Some(topic) = metadata_cache.get_topic_by_name(&topic_name) {
            write_topic_data(
                message_storage_adapter.clone(),
                topic_name,
                topic.topic_id,
                record,
            )
            .await;
        } else {
            error!(
                "When writing a message to the system topic {}, the topic was found not to exist",
                topic_name
            );
        }
    }
}

async fn build_node_version() {
    
}

async fn build_node_cluster(client_poll: Arc<ClientPool>) -> Option<Record> {
    let conf = broker_mqtt_conf();
    let request = NodeListRequest {
        cluster_name: conf.cluster_name.clone(),
    };
    match node_list(client_poll, conf.placement_center.clone(), request).await {
        Ok(results) => {
            let mut node_list = Vec::new();
            for node in results.nodes {
                match serde_json::from_slice::<BrokerNode>(&node) {
                    Ok(data) => node_list.push(data),
                    Err(e) => {
                        error!("Retrieving cluster Node list, parsing Node information failed, error message :{}",e.to_string());
                    }
                }
            }

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
            return MQTTMessage::build_system_topic_message(
                SYSTEM_TOPIC_BROKERS.to_string(),
                content,
            );
        }
        Err(e) => {
            error!(
                "Failed to get cluster Node list with error message : {}",
                e.to_string()
            );
            return None;
        }
    }
}
