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

use common_base::tools::now_second;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::PublishProperties;

use super::cache::MQTTCacheManager;

pub fn is_message_expire(message: &MqttMessage) -> bool {
    message.expiry_interval < now_second()
}

pub async fn build_message_expire(
    cache_manager: &Arc<MQTTCacheManager>,
    publish_properties: &Option<PublishProperties>,
) -> u64 {
    if let Some(properties) = publish_properties {
        if let Some(expire) = properties.message_expiry_interval {
            if expire > 0 {
                return now_second() + expire as u64;
            }
        }
    }

    let cluster = cache_manager.broker_cache.get_cluster_config().await;
    now_second() + cluster.mqtt_protocol_config.max_message_expiry_interval
}

#[cfg(test)]
mod tests {
    use crate::handler::message::{build_message_expire, is_message_expire};
    use crate::handler::tool::test_build_mqtt_cache_manager;
    use common_base::tools::now_second;
    use common_config::config::{BrokerConfig, MqttProtocolConfig};
    use metadata_struct::mqtt::message::MqttMessage;
    use protocol::mqtt::common::PublishProperties;

    #[tokio::test]
    async fn build_message_expire_test() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let cluster = BrokerConfig {
            mqtt_protocol_config: MqttProtocolConfig {
                max_message_expiry_interval: 10,
                ..Default::default()
            },
            ..Default::default()
        };
        cache_manager.broker_cache.set_cluster_config(cluster).await;

        let publish_properties = None;
        let res = build_message_expire(&cache_manager, &publish_properties).await;
        assert_eq!(res, now_second() + 10);

        let publish_properties = PublishProperties {
            message_expiry_interval: Some(3),
            ..Default::default()
        };
        let res = build_message_expire(&cache_manager, &Some(publish_properties)).await;
        assert_eq!(res, now_second() + 3);
    }

    #[test]
    fn is_message_expire_test() {
        let message = MqttMessage {
            expiry_interval: now_second() - 10,
            ..Default::default()
        };

        assert!(is_message_expire(&message));

        let message = MqttMessage {
            expiry_interval: now_second() + 10,
            ..Default::default()
        };

        assert!(!is_message_expire(&message));
    }
}
