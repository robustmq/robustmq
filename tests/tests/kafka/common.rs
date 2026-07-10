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

use admin_server::client::AdminHttpClient;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::BaseConsumer;

pub fn bootstrap_servers() -> String {
    std::env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or_else(|_| "localhost:9092".to_string())
}

pub fn consumer() -> BaseConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers())
        .create()
        .expect("create kafka consumer")
}

pub fn admin_http_addr() -> String {
    std::env::var("ROBUSTMQ_ADMIN_ADDR").unwrap_or_else(|_| "http://127.0.0.1:58080".to_string())
}

// Toggle the cluster-level Kafka `auto_create_topics_enable` dynamic config
// through the admin HTTP API.
pub async fn set_auto_create_topics(enabled: bool) {
    let client = AdminHttpClient::new(&admin_http_addr());
    let body = serde_json::json!({
        "config_type": "KafkaDynamic",
        "config": format!("{{\"auto_create_topics_enable\":{}}}", enabled),
    });
    client
        .post_raw("/api/cluster/config/set", &body)
        .await
        .expect("set KafkaDynamic auto_create_topics_enable");
}
