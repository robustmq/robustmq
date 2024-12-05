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

use bytes::Bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::lastwill::LastWillData;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{LastWill, LastWillProperties, Publish, PublishProperties};
use storage_adapter::storage::StorageAdapter;

use super::cache::CacheManager;
use super::error::MqttBrokerError;
use super::message::build_message_expire;
use super::retain::save_retain_message;
use super::topic::try_init_topic;
use crate::storage::message::MessageStorage;
use crate::storage::session::SessionStorage;

pub async fn send_last_will_message<S>(
    client_id: &str,
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    message_storage_adapter: Arc<S>,
) -> Result<(), MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let (topic_name, publish_res, publish_properties) =
        build_publish_message_by_lastwill(last_will, last_will_properties)?;

    if publish_res.is_none() || topic_name.is_empty() {
        // If building a publish message from lastwill fails, the message is ignored without throwing an error.
        return Ok(());
    }

    let publish = publish_res.unwrap();

    let topic = try_init_topic(
        &topic_name,
        cache_manager,
        &message_storage_adapter,
        client_pool,
    )
    .await?;

    save_retain_message(
        cache_manager,
        client_pool,
        topic_name,
        client_id,
        &publish,
        &publish_properties,
    )
    .await?;

    // Persisting stores message data
    let message_storage = MessageStorage::new(message_storage_adapter.clone());

    let message_expire = build_message_expire(cache_manager, &publish_properties);
    if let Some(record) =
        MqttMessage::build_record(client_id, &publish, &publish_properties, message_expire)
    {
        message_storage
            .append_topic_message(&topic.topic_id, vec![record])
            .await?;
    }
    Ok(())
}

fn build_publish_message_by_lastwill(
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
) -> Result<(String, Option<Publish>, Option<PublishProperties>), MqttBrokerError> {
    if let Some(will) = last_will {
        if will.topic.is_empty() || will.message.is_empty() {
            return Ok(("".to_string(), None, None));
        }

        let topic_name = String::from_utf8(will.topic.to_vec())?;

        let publish = Publish {
            dup: false,
            qos: will.qos,
            pkid: 0,
            retain: will.retain,
            topic: Bytes::from(topic_name.clone()),
            payload: will.message.clone(),
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
        return Ok((topic_name, Some(publish), properties));
    }

    Ok(("".to_string(), None, None))
}

pub async fn save_last_will_message(
    client_id: String,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), MqttBrokerError> {
    if last_will.is_none() {
        return Ok(());
    }

    let session_storage = SessionStorage::new(client_pool.clone());
    let lastwill = LastWillData {
        client_id: client_id.clone(),
        last_will: last_will.clone(),
        last_will_properties: last_will_properties.clone(),
    };

    session_storage
        .save_last_will_message(client_id, lastwill.encode())
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
        let res = build_publish_message_by_lastwill(&None, &None).unwrap();
        assert!(res.0.is_empty());
        assert!(res.1.is_none());
        assert!(res.2.is_none());

        let topic = "t1".to_string();
        let message = "message".to_string();

        let lastwill = LastWill {
            topic: Bytes::from(topic.clone()),
            message: Bytes::from(message.clone()),
            qos: protocol::mqtt::common::QoS::AtLeastOnce,
            retain: false,
        };

        let lastwill_properties = None;
        let (t, p, pp) =
            build_publish_message_by_lastwill(&Some(lastwill.clone()), &lastwill_properties)
                .unwrap();
        assert_eq!(t, topic);
        let p_tmp = p.unwrap();
        assert_eq!(p_tmp.topic, Bytes::from(topic.clone()));
        assert_eq!(p_tmp.payload, Bytes::from(message.clone()));
        assert_eq!(p_tmp.qos, protocol::mqtt::common::QoS::AtLeastOnce);
        assert!(!p_tmp.retain);
        assert_eq!(p_tmp.pkid, 0);
        assert!(!p_tmp.dup);
        assert!(pp.is_none());

        let lastwill_properties = Some(LastWillProperties {
            delay_interval: Some(1),
            payload_format_indicator: Some(2),
            message_expiry_interval: Some(3),
            content_type: Some("t1".to_string()),
            response_topic: Some("t2".to_string()),
            correlation_data: Some(Bytes::from("t3".to_string())),
            user_properties: Vec::new(),
        });

        let (t, p, pp) =
            build_publish_message_by_lastwill(&Some(lastwill), &lastwill_properties).unwrap();
        assert_eq!(t, topic);
        let p_tmp = p.unwrap();
        assert_eq!(p_tmp.topic, Bytes::from(topic.clone()));
        assert_eq!(p_tmp.payload, Bytes::from(message.clone()));
        assert_eq!(p_tmp.qos, protocol::mqtt::common::QoS::AtLeastOnce);
        assert!(!p_tmp.retain);
        assert_eq!(p_tmp.pkid, 0);
        assert!(!p_tmp.dup);
        assert!(pp.is_some());

        let pp_tmp = pp.unwrap();
        assert_eq!(pp_tmp.payload_format_indicator.unwrap(), 2);
        assert_eq!(pp_tmp.message_expiry_interval.unwrap(), 3);
        assert!(pp_tmp.topic_alias.is_none());
        assert_eq!(pp_tmp.response_topic.unwrap(), "t2".to_string());
        assert_eq!(
            pp_tmp.correlation_data.unwrap(),
            Bytes::from("t3".to_string())
        );
        assert!(pp_tmp.user_properties.is_empty());
        assert!(pp_tmp.subscription_identifiers.is_empty());
        assert_eq!(pp_tmp.content_type.unwrap(), "t1".to_string())
    }
}
