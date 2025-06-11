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

use crate::handler::cache::CacheManager;
use crate::observability::slow::sub::{read_slow_sub_record, SlowSubData};
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::utils::file_utils::get_project_root;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ListSlowSubScribeRaw, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListSystemAlarmRaw,
    ListSystemAlarmReply, ListSystemAlarmRequest, SetSystemAlarmConfigReply,
    SetSystemAlarmConfigRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

// ---- slow subscribe ----
pub async fn list_slow_subscribe_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<ListSlowSubscribeRequest>,
) -> Result<Response<ListSlowSubscribeReply>, Status> {
    let list_slow_subscribe_request = request.into_inner();
    let mut list_slow_subscribe_raw: Vec<ListSlowSubScribeRaw> = Vec::new();
    let mqtt_config = broker_mqtt_conf();
    if cache_manager.get_slow_sub_config().enable {
        let path = mqtt_config.log.log_path.clone();
        let path_buf = get_project_root()?.join(path.replace("./", "") + "/slow_sub.log");
        let deque = read_slow_sub_record(list_slow_subscribe_request, path_buf)?;
        for slow_sub_data in deque {
            match serde_json::from_str::<SlowSubData>(slow_sub_data.as_str()) {
                Ok(data) => {
                    let raw = ListSlowSubScribeRaw {
                        client_id: data.client_id,
                        topic: data.topic,
                        time_ms: data.time_ms,
                        node_info: data.node_info,
                        create_time: data.create_time,
                        sub_name: data.sub_name,
                    };
                    list_slow_subscribe_raw.push(raw);
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }
    Ok(Response::new(ListSlowSubscribeReply {
        list_slow_subscribe_raw,
    }))
}

pub async fn set_system_alarm_config_by_req(
    cache_manager: &Arc<CacheManager>,
    req: &SetSystemAlarmConfigRequest,
) -> Result<SetSystemAlarmConfigReply, Status> {
    let mut system_monitor_config = cache_manager.get_system_monitor_config();
    system_monitor_config.enable = req.enable;
    if let Some(os_cpu_high_watermark) = req.os_cpu_high_watermark {
        system_monitor_config.os_cpu_high_watermark = os_cpu_high_watermark;
    }
    if let Some(os_cpu_low_watermark) = req.os_cpu_low_watermark {
        system_monitor_config.os_cpu_low_watermark = os_cpu_low_watermark;
    }
    if let Some(os_memory_high_watermark) = req.os_memory_high_watermark {
        system_monitor_config.os_memory_high_watermark = os_memory_high_watermark;
    }
    cache_manager
        .update_system_monitor_config(system_monitor_config)
        .await?;
    Ok(SetSystemAlarmConfigReply {
        enable: req.enable,
        os_cpu_high_watermark: req.os_cpu_high_watermark,
        os_cpu_low_watermark: req.os_cpu_low_watermark,
        os_memory_high_watermark: req.os_memory_high_watermark,
    })
}

pub async fn list_system_alarm_by_req(
    cache_manager: &Arc<CacheManager>,
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
    use crate::handler::cluster_config::build_default_cluster_config;
    use crate::observability::system_topic::sysmon::SystemAlarmEventMessage;
    use crate::storage::message::cluster_name;
    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use grpc_clients::pool::ClientPool;

    #[tokio::test]
    pub async fn test_set_system_alarm_config_by_req() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let cache_client_pool = Arc::new(ClientPool::new(3));
        let cache_manager = Arc::new(CacheManager::new(cache_client_pool, cluster_name()));
        cache_manager.set_cluster_info(build_default_cluster_config());
        let req = SetSystemAlarmConfigRequest {
            enable: true,
            os_cpu_high_watermark: Some(80.0),
            os_cpu_low_watermark: Some(20.0),
            os_memory_high_watermark: Some(75.0),
        };
        let reply = set_system_alarm_config_by_req(&cache_manager, &req)
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to set system alarm config: {}", e);
            });

        assert_eq!(reply.enable, req.enable);
        assert_eq!(reply.os_cpu_high_watermark, req.os_cpu_high_watermark);
        assert_eq!(reply.os_cpu_low_watermark, req.os_cpu_low_watermark);
        assert_eq!(reply.os_memory_high_watermark, req.os_memory_high_watermark);
    }

    #[tokio::test]
    pub async fn test_list_system_alarm_by_req() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);
        let cache_client_pool = Arc::new(ClientPool::new(3));
        let cache_manager = Arc::new(CacheManager::new(cache_client_pool, cluster_name()));
        cache_manager.set_cluster_info(build_default_cluster_config());

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
                panic!("Failed to list system alarms: {}", e);
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
