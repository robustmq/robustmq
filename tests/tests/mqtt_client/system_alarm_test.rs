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

use crate::mqtt_protocol::common::broker_grpc_addr;
use common_config::broker::{default_broker_config, init_broker_conf_by_config};
use grpc_clients::mqtt::admin::call::mqtt_broker_set_system_alarm_config;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::{
    SetSystemAlarmConfigReply, SetSystemAlarmConfigRequest,
};
use std::sync::Arc;

#[tokio::test]
pub async fn set_cluster_system_alarm_config() {
    let config = default_broker_config();
    init_broker_conf_by_config(config.clone());

    let client_pool = Arc::new(ClientPool::new(3));
    let grpc_addr = vec![broker_grpc_addr()];

    let is_enables = [true, false];
    for is_enable in is_enables.iter() {
        let valid_request = SetSystemAlarmConfigRequest {
            enable: Some(*is_enable),
            os_cpu_high_watermark: Some(90.0),
            os_cpu_low_watermark: Some(40.0),
            os_memory_high_watermark: None,
            os_cpu_check_interval_ms: None,
        };

        let expect_reply = SetSystemAlarmConfigReply {
            enable: *is_enable,
            os_cpu_high_watermark: Some(
                valid_request
                    .os_cpu_high_watermark
                    .unwrap_or(config.mqtt_system_monitor.os_cpu_high_watermark),
            ),
            os_cpu_low_watermark: Some(
                valid_request
                    .os_cpu_low_watermark
                    .unwrap_or(config.mqtt_system_monitor.os_cpu_low_watermark),
            ),
            os_memory_high_watermark: Some(
                valid_request
                    .os_memory_high_watermark
                    .unwrap_or(config.mqtt_system_monitor.os_memory_high_watermark),
            ),
            os_cpu_check_interval_ms: Some(
                valid_request
                    .os_cpu_check_interval_ms
                    .unwrap_or(config.mqtt_system_monitor.os_cpu_check_interval_ms),
            ),
        };

        match mqtt_broker_set_system_alarm_config(&client_pool, &grpc_addr, valid_request).await {
            Ok(data) => {
                assert_eq!(expect_reply, data);
            }

            Err(e) => {
                eprintln!("Failed enable_offline_message: {e:?}");
                std::process::exit(1);
            }
        }
    }
}

#[tokio::test]
pub async fn list_cluster_system_alarm() {
    let client_pool = Arc::new(ClientPool::new(3));
    let grpc_addr = vec![broker_grpc_addr()];

    let request = protocol::broker_mqtt::broker_mqtt_admin::ListSystemAlarmRequest {};

    match grpc_clients::mqtt::admin::call::mqtt_broker_list_system_alarm(
        &client_pool,
        &grpc_addr,
        request,
    )
    .await
    {
        Ok(data) => {
            assert!(!data.list_system_alarm_raw.is_empty());
        }
        Err(e) => {
            eprintln!("Failed to list system alarm: {e:?}");
            std::process::exit(1);
        }
    }
}
