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

use super::{
    cache::MQTTCacheManager,
    delay_message::{save_delay_message, DelayPublishTopic},
    error::MqttBrokerError,
    message::build_message_expire,
};
use crate::{
    core::{qos::save_temporary_qos2_message, retain::save_retain_message},
    storage::message::MessageStorage,
    subscribe::manager::SubscribeManager,
};
use common_metrics::mqtt::publish::record_messages_dropped_no_subscribers_incr;
use delay_message::manager::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    mqtt::topic::Topic,
    storage::{
        adapter_record::AdapterWriteRecord,
        record::{StorageRecordProtocolData, StorageRecordProtocolDataMqtt},
    },
};
use protocol::mqtt::common::{Publish, PublishProperties, QoS};
use storage_adapter::driver::StorageDriverManager;

pub fn is_exist_subscribe(
    subscribe_manager: &Arc<SubscribeManager>,
    tenant: &str,
    topic: &str,
) -> bool {
    subscribe_manager
        .topic_subscribes
        .get(tenant)
        .map(|t| t.contains_key(topic))
        .unwrap_or(false)
}

#[derive(Clone)]
pub struct SaveMessageContext {
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub publish: Publish,
    pub publish_properties: Option<PublishProperties>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub client_id: String,
    pub topic: Topic,
    pub delay_info: Option<DelayPublishTopic>,
}

pub async fn save_message(context: SaveMessageContext) -> Result<Option<String>, MqttBrokerError> {
    // A delayed publish must not be reported as accepted when policy prevents queuing it.
    let offline_message_disabled = !context
        .cache_manager
        .node_cache
        .get_cluster_config()
        .mqtt_runtime
        .offline_message
        .enable;
    let not_exist_subscribe = !is_exist_subscribe(
        &context.subscribe_manager,
        &context.topic.tenant,
        &context.topic.topic_name,
    );
    if should_reject_delayed_publish(
        offline_message_disabled,
        not_exist_subscribe,
        context.delay_info.is_some(),
    ) {
        record_messages_dropped_no_subscribers_incr();
        return Err(MqttBrokerError::DelayedMessageRejectedNoSubscribers(
            context.topic.topic_name.clone(),
        ));
    }

    // Whether or not offline messages are enabled
    // persistent storage must be used to retain the messages.
    save_retain_message(
        &context.storage_driver_manager,
        &context.cache_manager,
        &context.topic.tenant,
        &context.topic.topic_name,
        &context.publish,
        &context.publish_properties,
    )
    .await?;

    // offline message
    if offline_message_disabled && not_exist_subscribe {
        record_messages_dropped_no_subscribers_incr();
        return Ok(None);
    }

    // save delay message
    if let Some(delay_info) = &context.delay_info {
        return save_delay_message(
            &context.delay_message_manager,
            &context.topic.tenant,
            &context.publish.payload,
            delay_info,
        )
        .await;
    }

    // save message
    let message_expire =
        build_message_expire(&context.cache_manager, &context.publish_properties).await;
    let mqtt_data = build_mqtt_protocol_data(
        &context.client_id,
        &context.publish,
        &context.publish_properties,
    )
    .await;

    let record = AdapterWriteRecord::new(
        context.topic.topic_name.clone(),
        context.publish.payload.clone(),
    )
    .with_protocol_data(Some(StorageRecordProtocolData {
        mqtt: Some(mqtt_data),
        ..Default::default()
    }))
    .with_expire_at(message_expire);

    save_simple_message(
        &context.storage_driver_manager,
        &context.client_id,
        &context.topic,
        &context.publish,
        &record,
    )
    .await
}

async fn save_simple_message(
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_id: &str,
    topic: &Topic,
    publish: &Publish,
    record: &AdapterWriteRecord,
) -> Result<Option<String>, MqttBrokerError> {
    let offsets = if publish.qos == QoS::ExactlyOnce {
        save_temporary_qos2_message(
            storage_driver_manager,
            record,
            &topic.tenant,
            &topic.topic_name,
            client_id,
            publish.p_kid,
        )
        .await?
    } else {
        let message_storage = MessageStorage::new(storage_driver_manager.clone());
        message_storage
            .append_topic_message(&topic.tenant, &topic.topic_name, vec![record.clone()])
            .await?
    };

    Ok(Some(format!("{offsets:?}")))
}

fn should_reject_delayed_publish(
    offline_message_disabled: bool,
    not_exist_subscribe: bool,
    is_delayed_publish: bool,
) -> bool {
    offline_message_disabled && not_exist_subscribe && is_delayed_publish
}

pub async fn build_mqtt_protocol_data(
    client_id: &str,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> StorageRecordProtocolDataMqtt {
    if let Some(properties) = publish_properties {
        StorageRecordProtocolDataMqtt {
            client_id: client_id.to_string(),
            retain: publish.retain,
            format_indicator: properties.payload_format_indicator,
            response_topic: properties.response_topic.clone(),
            correlation_data: properties.correlation_data.clone(),
            content_type: properties.content_type.clone(),
            user_properties: properties.user_properties.clone(),
        }
    } else {
        StorageRecordProtocolDataMqtt {
            client_id: client_id.to_string(),
            retain: publish.retain,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_mqtt_protocol_data, should_reject_delayed_publish};
    use bytes::Bytes;
    use protocol::mqtt::common::{Publish, PublishProperties};

    #[test]
    fn reject_delayed_publish_only_when_offline_storage_is_disabled_and_no_subscribers_exist() {
        assert!(should_reject_delayed_publish(true, true, true));
        assert!(!should_reject_delayed_publish(false, true, true));
        assert!(!should_reject_delayed_publish(true, false, true));
        assert!(!should_reject_delayed_publish(true, true, false));
    }

    #[tokio::test]
    async fn build_mqtt_protocol_data_preserves_publish_properties() {
        let publish = Publish {
            retain: true,
            ..Publish::new("topic", "payload", true)
        };
        let properties = PublishProperties {
            payload_format_indicator: Some(1),
            response_topic: Some("response/topic".to_string()),
            correlation_data: Some(Bytes::from_static(b"correlation")),
            content_type: Some("application/json".to_string()),
            user_properties: vec![("trace".to_string(), "abc".to_string())],
            ..Default::default()
        };

        let data = build_mqtt_protocol_data("client", &publish, &Some(properties)).await;

        assert_eq!(data.client_id, "client");
        assert!(data.retain);
        assert_eq!(data.format_indicator, Some(1));
        assert_eq!(data.response_topic.as_deref(), Some("response/topic"));
        assert_eq!(
            data.correlation_data,
            Some(Bytes::from_static(b"correlation"))
        );
        assert_eq!(data.content_type.as_deref(), Some("application/json"));
        assert_eq!(
            data.user_properties,
            vec![("trace".to_string(), "abc".to_string())]
        );
    }

    #[tokio::test]
    async fn build_mqtt_protocol_data_uses_defaults_without_properties() {
        let publish = Publish::new("topic", "payload", false);
        let data = build_mqtt_protocol_data("client", &publish, &None).await;

        assert_eq!(data.client_id, "client");
        assert!(!data.retain);
        assert_eq!(data.format_indicator, None);
        assert_eq!(data.response_topic, None);
        assert_eq!(data.correlation_data, None);
        assert_eq!(data.content_type, None);
        assert!(data.user_properties.is_empty());
    }
}
