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

use super::cache::MQTTCacheManager;
use super::error::MqttBrokerError;
use super::topic::try_init_topic;
use crate::core::message::build_message_expire;
use crate::core::offline_message::build_mqtt_protocol_data;
use crate::core::{retain::save_retain_message, tool::ResultMqttBrokerError};
use crate::storage::last_will::LastWillStorage;
use crate::storage::message::MessageStorage;
use bytes::Bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::record::StorageRecordProtocolData;
use protocol::mqtt::common::{LastWill, LastWillProperties, Publish, PublishProperties};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

pub async fn send_last_will_message(
    cache_manager: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    last_will: &MqttLastWillData,
) -> ResultMqttBrokerError {
    let will_data = if let Some(data) = last_will.last_will.clone() {
        data
    } else {
        return Ok(());
    };

    if will_data.topic.is_empty() || will_data.message.is_empty() {
        return Ok(());
    }

    // init topic
    let topic_name = String::from_utf8(will_data.topic.to_vec())?;
    let topic = try_init_topic(
        &last_will.tenant,
        &topic_name,
        cache_manager,
        storage_driver_manager,
        client_pool,
    )
    .await?;

    // build params
    let (publish, publish_properties) =
        build_publish_message_by_lastwill(&topic_name, &will_data, &last_will.last_will_properties)
            .await?;

    // save retain message
    save_retain_message(
        storage_driver_manager,
        cache_manager,
        &last_will.tenant,
        &topic_name,
        &publish,
        &publish_properties,
    )
    .await?;

    // save message
    let mqtt_data =
        build_mqtt_protocol_data(&last_will.client_id, &publish, &publish_properties).await;

    let message_expire = build_message_expire(cache_manager, &publish_properties).await;
    let record = AdapterWriteRecord::new(topic_name.to_string(), publish.payload.clone())
        .with_protocol_data(Some(StorageRecordProtocolData {
            mqtt: Some(mqtt_data),
            nats: None,
            mq9: None,
        }))
        .with_expire_at(message_expire);

    let message_storage = MessageStorage::new(storage_driver_manager.clone());
    message_storage
        .append_topic_message(&topic.tenant, &topic.topic_name, vec![record])
        .await?;

    Ok(())
}

async fn build_publish_message_by_lastwill(
    topic_name: &str,
    will: &LastWill,
    last_will_properties: &Option<LastWillProperties>,
) -> Result<(Publish, Option<PublishProperties>), MqttBrokerError> {
    let publish = Publish {
        dup: false,
        qos: will.qos,
        retain: will.retain,
        topic: Bytes::from(topic_name.to_string()),
        payload: will.message.clone(),
        p_kid: 0,
    };

    let properties = last_will_properties
        .as_ref()
        .map(|properties| PublishProperties {
            payload_format_indicator: properties.payload_format_indicator,
            message_expiry_interval: properties.message_expiry_interval,
            topic_alias: None,
            response_topic: properties.response_topic.clone(),
            user_properties: Vec::new(),
            subscription_identifiers: Vec::new(),
            correlation_data: properties.correlation_data.clone(),
            content_type: properties.content_type.clone(),
        });
    Ok((publish, properties))
}

pub async fn save_last_will_message(
    tenant: &str,
    client_id: &str,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> ResultMqttBrokerError {
    if last_will.is_none() {
        return Ok(());
    }

    let last_will_storage = LastWillStorage::new(storage_driver_manager.clone());
    let lastwill = MqttLastWillData {
        tenant: tenant.to_string(),
        client_id: client_id.to_string(),
        last_will: last_will.clone(),
        last_will_properties: last_will_properties.clone(),
    };

    last_will_storage
        .save_last_will_message(tenant, client_id, &lastwill)
        .await?;

    Ok(())
}

pub fn last_will_delay_interval(last_will_properties: &Option<LastWillProperties>) -> Option<u64> {
    let delay_interval = if let Some(properties) = last_will_properties.clone() {
        properties.delay_interval?
    } else {
        return None;
    };

    Some(delay_interval as u64)
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use protocol::mqtt::common::{LastWill, LastWillProperties};

    use super::{build_publish_message_by_lastwill, last_will_delay_interval};

    #[tokio::test]
    pub async fn last_will_delay_interval_test() {
        let res = last_will_delay_interval(&None);
        assert!(res.is_none());

        let last_will_properties = LastWillProperties::default();
        let res = last_will_delay_interval(&Some(last_will_properties));
        assert!(res.is_none());

        let last_will_properties = LastWillProperties {
            delay_interval: Some(10),
            ..Default::default()
        };
        let res = last_will_delay_interval(&Some(last_will_properties));
        assert_eq!(res.unwrap(), 10);
    }

    #[tokio::test]
    pub async fn build_publish_message_by_lastwill_test() {
        let topic = "t1".to_string();
        let message = "message".to_string();

        let lastwill = LastWill {
            topic: Bytes::from(topic.clone()),
            message: Bytes::from(message.clone()),
            qos: protocol::mqtt::common::QoS::AtLeastOnce,
            retain: false,
        };

        // No properties
        let (publish, properties) = build_publish_message_by_lastwill(&topic, &lastwill, &None)
            .await
            .unwrap();
        assert_eq!(publish.topic, Bytes::from(topic.clone()));
        assert_eq!(publish.payload, Bytes::from(message.clone()));
        assert_eq!(publish.qos, protocol::mqtt::common::QoS::AtLeastOnce);
        assert!(!publish.retain);
        assert_eq!(publish.p_kid, 0);
        assert!(!publish.dup);
        assert!(properties.is_none());

        // With properties
        let lastwill_properties = Some(LastWillProperties {
            delay_interval: Some(1),
            payload_format_indicator: Some(2),
            message_expiry_interval: Some(3),
            content_type: Some("t1".to_string()),
            response_topic: Some("t2".to_string()),
            correlation_data: Some(Bytes::from("t3".to_string())),
            user_properties: Vec::new(),
        });

        let (publish, properties) =
            build_publish_message_by_lastwill(&topic, &lastwill, &lastwill_properties)
                .await
                .unwrap();
        assert_eq!(publish.topic, Bytes::from(topic.clone()));
        assert_eq!(publish.payload, Bytes::from(message.clone()));
        assert_eq!(publish.qos, protocol::mqtt::common::QoS::AtLeastOnce);
        assert!(!publish.retain);
        assert_eq!(publish.p_kid, 0);
        assert!(!publish.dup);
        assert!(properties.is_some());

        let pp = properties.unwrap();
        assert_eq!(pp.payload_format_indicator.unwrap(), 2);
        assert_eq!(pp.message_expiry_interval.unwrap(), 3);
        assert!(pp.topic_alias.is_none());
        assert_eq!(pp.response_topic.unwrap(), "t2".to_string());
        assert_eq!(pp.correlation_data.unwrap(), Bytes::from("t3".to_string()));
        assert!(pp.user_properties.is_empty());
        assert!(pp.subscription_identifiers.is_empty());
        assert_eq!(pp.content_type.unwrap(), "t1".to_string());
    }
}
