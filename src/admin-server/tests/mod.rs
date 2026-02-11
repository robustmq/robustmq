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

// #[cfg(test)]
// mod tests {
//     const ADMIN_SERVER_ADDR: &str = "http://127.0.0.1:8080";
//     const CLUSTER_CONFIG_SET_PATH: &str = "/api/cluster/config/set";
//     const CLUSTER_CONFIG_GET_PATH: &str = "/api/cluster/config/get";

//     async fn set_config(config_type: &str, config: &str) -> reqwest::StatusCode {
//         let client = reqwest::Client::new();
//         // config 中的 JSON 需要转义内部引号
//         let escaped_config = config.replace('"', "\\\"");
//         let body = format!(
//             r#"{{"config_type":"{}","config":"{}"}}"#,
//             config_type, escaped_config
//         );

//         let response = client
//             .post(format!("{}{}", ADMIN_SERVER_ADDR, CLUSTER_CONFIG_SET_PATH))
//             .header("Content-Type", "application/json")
//             .body(body)
//             .send()
//             .await
//             .expect("Failed to send request");

//         response.status()
//     }

//     async fn get_full_config() -> serde_json::Value {
//         let client = reqwest::Client::new();
//         let response = client
//             .get(format!("{}{}", ADMIN_SERVER_ADDR, CLUSTER_CONFIG_GET_PATH))
//             .send()
//             .await
//             .expect("Failed to send request");
//         let text = response.text().await.expect("Failed to get body");
//         serde_json::from_str(&text).expect("Invalid JSON")
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_slow_subscribe() {
//         let config = r#"{"enable":true,"record_time":60,"delay_type":"Whole"}"#;
//         let status = set_config("SlowSubscribe", config).await;
//         assert_eq!(status, 200, "SlowSubscribe config set failed");

//         // 验证配置
//         let json = get_full_config().await;
//         let slow_sub = &json["data"]["mqtt_slow_subscribe_config"];
//         assert_eq!(slow_sub["enable"], true, "enable should be true");
//         assert_eq!(slow_sub["record_time"], 60, "record_time should be 60");
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_offline_message() {
//         let config = r#"{"enable":true,"expire_ms":3600000,"max_messages_num":1000}"#;
//         let status = set_config("OfflineMessage", config).await;
//         assert_eq!(status, 200, "OfflineMessage config set failed");

//         // 验证配置
//         let json = get_full_config().await;
//         let offline = &json["data"]["mqtt_offline_message"];
//         assert_eq!(offline["enable"], true, "enable should be true");
//         assert_eq!(offline["expire_ms"], 3600000);
//         assert_eq!(offline["max_messages_num"], 1000);
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_flapping_detect() {
//         let config =
//             r#"{"enable":true,"window_time":60,"max_client_connections":100,"ban_time":300}"#;
//         let status = set_config("FlappingDetect", config).await;
//         assert_eq!(status, 200, "FlappingDetect config set failed");

//         // 验证配置
//         let json = get_full_config().await;
//         let flapping = &json["data"]["mqtt_flapping_detect"];
//         assert_eq!(flapping["enable"], true, "enable should be true");
//         assert_eq!(flapping["window_time"], 60);
//         assert_eq!(flapping["max_client_connections"], 100);
//         assert_eq!(flapping["ban_time"], 300);
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_system_alarm() {
//         let config =
//             r#"{"enable":true,"os_cpu_high_watermark":80.0,"os_memory_high_watermark":85.0}"#;
//         let status = set_config("SystemAlarm", config).await;
//         assert_eq!(status, 200, "SystemAlarm config set failed");

//         // 验证配置
//         let json = get_full_config().await;
//         let alarm = &json["data"]["mqtt_system_monitor"];
//         assert_eq!(alarm["enable"], true, "enable should be true");
//         assert_eq!(alarm["os_cpu_high_watermark"], 80.0);
//         assert_eq!(alarm["os_memory_high_watermark"], 85.0);
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_mqtt_protocol() {
//         let config = r#"{"max_session_expiry_interval":86400,"default_session_expiry_interval":3600,"topic_alias_max":100,"max_qos_flight_message":100,"max_packet_size":1048576,"receive_max":1000,"max_message_expiry_interval":604800,"client_pkid_persistent":true}"#;
//         let status = set_config("MqttProtocol", config).await;
//         assert_eq!(status, 200, "MqttProtocol config set failed");

//         // 验证配置
//         let json = get_full_config().await;
//         let protocol = &json["data"]["mqtt_protocol_config"];
//         assert_eq!(protocol["max_session_expiry_interval"], 86400);
//         assert_eq!(protocol["max_packet_size"], 1048576);
//         assert_eq!(protocol["client_pkid_persistent"], true);
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_mqtt_security() {
//         let config = r#"{"is_self_protection_status":false,"secret_free_login":true}"#;
//         let status = set_config("MqttSecurity", config).await;
//         assert_eq!(status, 200, "MqttSecurity config set failed");

//         // 验证配置
//         let json = get_full_config().await;
//         let security = &json["data"]["mqtt_security"];
//         assert_eq!(security["is_self_protection_status"], false);
//         assert_eq!(security["secret_free_login"], true);
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_mqtt_schema() {
//         let config = r#"{"enable":true}"#;
//         let status = set_config("MqttSchema", config).await;
//         assert_eq!(status, 200, "MqttSchema config set failed");

//         // 验证配置
//         let json = get_full_config().await;
//         let schema = &json["data"]["mqtt_schema"];
//         assert_eq!(schema["enable"], true, "enable should be true");
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_invalid_feature_type() {
//         let config = r#"{"enable":true}"#;
//         let status = set_config("InvalidType", config).await;
//         // Invalid type returns 200 with error message in body
//         assert_eq!(status, 200, "Expected 200 for invalid feature type");
//     }

//     #[tokio::test]
//     async fn test_cluster_config_set_invalid_json_config() {
//         let status = set_config("SlowSubscribe", "not valid json").await;
//         // Invalid config JSON returns 200 with error message in body
//         assert_eq!(status, 200, "Expected 200 for invalid config JSON");
//     }
// }
