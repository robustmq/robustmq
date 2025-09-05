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

use crate::handler::cache::MQTTCacheManager;
use std::str::FromStr;

use crate::common::metrics_cache::MetricsCacheManager;
use common_base::enum_type::delay_type::DelayType;
use protocol::broker::broker_mqtt_admin::{
    ListSlowSubScribeRaw, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListSystemAlarmRaw,
    ListSystemAlarmReply, ListSystemAlarmRequest, SetSlowSubscribeConfigReply,
    SetSlowSubscribeConfigRequest, SetSystemAlarmConfigReply, SetSystemAlarmConfigRequest,
};
use std::sync::Arc;
use tonic::Status;

// ---- slow subscribe ----
pub async fn set_slow_subscribe_config_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    request: &SetSlowSubscribeConfigRequest,
) -> Result<SetSlowSubscribeConfigReply, crate::handler::error::MqttBrokerError> {
    let mut slow_subscribe_config = cache_manager.get_slow_sub_config();
    if let Ok(delay_type) = DelayType::from_str(request.delay_type.as_str()) {
        slow_subscribe_config.delay_type = delay_type
    }
    slow_subscribe_config.max_store_num = request.max_store_num;
    cache_manager.update_slow_sub_config(slow_subscribe_config.clone());
    Ok(SetSlowSubscribeConfigReply {
        is_enable: slow_subscribe_config.enable,
        delay_type: slow_subscribe_config.delay_type.to_string(),
        max_store_num: slow_subscribe_config.max_store_num,
    })
}

pub async fn list_slow_subscribe_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    metrics_cache_manager: &Arc<MetricsCacheManager>,
    _request: &ListSlowSubscribeRequest,
) -> Result<ListSlowSubscribeReply, crate::handler::error::MqttBrokerError> {
    let mut list_slow_subscribe_raw: Vec<ListSlowSubScribeRaw> = Vec::new();
    if cache_manager.get_slow_sub_config().enable {
        for (_, slow_data) in metrics_cache_manager.slow_subscribe_info.iter_reverse() {
            list_slow_subscribe_raw.push(ListSlowSubScribeRaw {
                client_id: slow_data.client_id.clone(),
                topic_name: slow_data.topic_name.clone(),
                time_span: slow_data.time_span,
                node_info: slow_data.node_info.clone(),
                create_time: slow_data.create_time,
                subscribe_name: slow_data.subscribe_name.clone(),
            });
        }
    }
    Ok(ListSlowSubscribeReply {
        list_slow_subscribe_raw,
    })
}

pub async fn set_system_alarm_config_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    req: &SetSystemAlarmConfigRequest,
) -> Result<SetSystemAlarmConfigReply, Status> {
    let mut system_monitor_config = cache_manager.get_system_monitor_config();
    if let Some(enable) = req.enable {
        system_monitor_config.enable = enable;
    }
    if let Some(os_cpu_high_watermark) = req.os_cpu_high_watermark {
        system_monitor_config.os_cpu_high_watermark = os_cpu_high_watermark;
    }
    if let Some(os_cpu_low_watermark) = req.os_cpu_low_watermark {
        system_monitor_config.os_cpu_low_watermark = os_cpu_low_watermark;
    }
    if let Some(os_memory_high_watermark) = req.os_memory_high_watermark {
        system_monitor_config.os_memory_high_watermark = os_memory_high_watermark;
    }
    if let Some(os_cpu_check_interval_ms) = req.os_cpu_check_interval_ms {
        system_monitor_config.os_cpu_check_interval_ms = os_cpu_check_interval_ms;
    }
    cache_manager.update_system_monitor_config(system_monitor_config.clone());
    Ok(SetSystemAlarmConfigReply {
        enable: system_monitor_config.enable,
        os_cpu_high_watermark: Some(system_monitor_config.os_cpu_high_watermark),
        os_cpu_low_watermark: Some(system_monitor_config.os_cpu_low_watermark),
        os_memory_high_watermark: Some(system_monitor_config.os_memory_high_watermark),
        os_cpu_check_interval_ms: Some(system_monitor_config.os_cpu_check_interval_ms),
    })
}

pub async fn list_system_alarm_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    _req: &ListSystemAlarmRequest,
) -> Result<ListSystemAlarmReply, Status> {
    let list_system_alarm_raw: Vec<ListSystemAlarmRaw> = cache_manager
        .alarm_events
        .iter()
        .map(|entry| {
            let system_alarm_message = entry.value();
            ListSystemAlarmRaw {
                name: system_alarm_message.name.clone(),
                message: system_alarm_message.message.clone(),
                activate_at: system_alarm_message.activate_at,
                activated: system_alarm_message.activated,
            }
        })
        .collect();

    Ok(ListSystemAlarmReply {
        list_system_alarm_raw,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::observability::system_topic::sysmon::SystemAlarmEventMessage;
    use common_config::broker::{broker_config, default_broker_config, init_broker_conf_by_config};
    use common_config::config::BrokerConfig;

    #[tokio::test]
    pub async fn test_set_system_alarm_config_by_req() {
        init_broker_conf_by_config(default_broker_config());
        let cache_manager = test_build_mqtt_cache_manager();
        cache_manager
            .broker_cache
            .set_cluster_config(default_broker_config());

        let req = SetSystemAlarmConfigRequest {
            enable: Some(true),
            os_cpu_high_watermark: Some(80.0),
            os_cpu_low_watermark: Some(20.0),
            os_memory_high_watermark: Some(75.0),
            os_cpu_check_interval_ms: None,
        };
        let reply = set_system_alarm_config_by_req(&cache_manager, &req)
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to set system alarm config: {e}");
            });

        let mqtt_conf = broker_config();
        assert_eq!(reply.enable, req.enable.unwrap());
        assert_eq!(reply.os_cpu_high_watermark, req.os_cpu_high_watermark);
        assert_eq!(reply.os_cpu_low_watermark, req.os_cpu_low_watermark);
        assert_eq!(reply.os_memory_high_watermark, req.os_memory_high_watermark);
        assert_eq!(
            reply.os_cpu_check_interval_ms,
            Some(mqtt_conf.mqtt_system_monitor.os_cpu_check_interval_ms)
        );
    }

    #[tokio::test]
    pub async fn test_list_system_alarm_by_req() {
        init_broker_conf_by_config(default_broker_config());
        let cache_manager = test_build_mqtt_cache_manager();
        cache_manager
            .broker_cache
            .set_cluster_config(BrokerConfig::default());

        let req = ListSystemAlarmRequest {};
        let test_event = "test_event";
        let message = SystemAlarmEventMessage {
            name: test_event.to_string(),
            message: test_event.to_string(),
            activate_at: 0,
            activated: false,
        };
        cache_manager.add_alarm_event(test_event.to_string(), message);
        let reply = list_system_alarm_by_req(&cache_manager, &req)
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to list system alarms: {e}");
            });

        assert!(!reply.list_system_alarm_raw.is_empty());
        assert_eq!(reply.list_system_alarm_raw.len(), 1);
        assert_eq!(reply.list_system_alarm_raw[0].name, test_event.to_string());
        assert_eq!(
            reply.list_system_alarm_raw[0].message,
            test_event.to_string()
        );
        assert_eq!(reply.list_system_alarm_raw[0].activate_at, 0);
        assert!(!reply.list_system_alarm_raw[0].activated);
    }
}
