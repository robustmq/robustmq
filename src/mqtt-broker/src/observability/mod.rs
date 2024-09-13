use std::sync::Arc;

use clients::poll::ClientPool;
use storage_adapter::storage::StorageAdapter;
use system_topic::SystemTopic;
use tokio::sync::broadcast;

use crate::handler::cache::CacheManager;

pub mod metrics;
pub mod slow;
pub mod system_topic;
pub mod warn;

pub async fn start_opservability<S>(
    cache_manager: Arc<CacheManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let system_topic = SystemTopic::new(
        cache_manager.clone(),
        message_storage_adapter.clone(),
        client_poll.clone(),
    );

    tokio::spawn(async move {
        system_topic.start_thread(stop_send).await;
    });

    
}
