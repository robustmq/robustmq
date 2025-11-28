use std::sync::Arc;

use crate::{
    handler::cache::MQTTCacheManager,
    storage::message::MessageStorage,
    subscribe::push::{build_pub_qos, build_sub_ids},
};
use dashmap::DashMap;
use storage_adapter::storage::ArcStorageAdapter;

use crate::{
    common::types::ResultMqttBrokerError,
    handler::error::MqttBrokerError,
    subscribe::{common::Subscriber, manager::SubscribeManager},
};

pub struct DirectlyPushManager {
    subscribe_manager: Arc<SubscribeManager>,
    cache_manager: Arc<MQTTCacheManager>,
    message_storage: ArcStorageAdapter,
    offset_cache: DashMap<String, u32>,
    uuid: String,
}

impl DirectlyPushManager {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<MQTTCacheManager>,
        message_storage: ArcStorageAdapter,
        uuid: String,
    ) -> Self {
        DirectlyPushManager {
            subscribe_manager,
            message_storage,
            offset_cache: DashMap::with_capacity(2),
            cache_manager,
            uuid,
        }
    }

    fn start(&self) -> ResultMqttBrokerError {
        if let Some(data) = self
            .subscribe_manager
            .directly_push
            .buckets_data_list
            .get(&self.uuid)
        {
            for row in data.iter() {
                let subscriber = row;
            }
        }

        Ok(())
    }

    fn next_message(&self) {}

    fn commit_offset(&self) {}

    async fn get_offset(
        &self,
        group: &str,
        subscriber: &Subscriber,
    ) -> Result<u32, MqttBrokerError> {
        if let Some(offset) = self.offset_cache.get(group) {
            return Ok(*offset);
        }

        let group_id = self.build_group_name(&subscriber);
        let qos = build_pub_qos(&self.cache_manager, &subscriber).await;
        let sub_ids = build_sub_ids(&subscriber);
        let storage = MessageStorage::new(self.message_storage.clone());
        let offset = storage
            .get_group_offset(group, &subscriber.topic_name)
            .await?;
        Ok(1)
    }

    fn build_group_name(&self, subscriber: &Subscriber) -> String {
        format!(
            "system_sub_{}_{}_{}",
            subscriber.client_id, subscriber.sub_path, subscriber.topic_name
        )
    }
}
