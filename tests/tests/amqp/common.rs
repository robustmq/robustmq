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

use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties};

pub fn amqp_broker_addr() -> String {
    std::env::var("AMQP_BROKER_ADDR").unwrap_or_else(|_| "127.0.0.1:25672".to_string())
}

fn amqp_user() -> String {
    std::env::var("AMQP_BROKER_USER").unwrap_or_else(|_| "admin".to_string())
}

fn amqp_password() -> String {
    std::env::var("AMQP_BROKER_PASSWORD").unwrap_or_else(|_| "robustmq".to_string())
}

fn amqp_uri() -> String {
    let mut uri = String::from("amqp://");
    uri.push_str(&amqp_user());
    uri.push(':');
    uri.push_str(&amqp_password());
    uri.push('@');
    uri.push_str(&amqp_broker_addr());
    uri.push('/');
    uri
}

pub async fn amqp_connection() -> Connection {
    Connection::connect(&amqp_uri(), ConnectionProperties::default())
        .await
        .expect("connect to AMQP broker")
}

pub async fn amqp_channel(conn: &Connection) -> Channel {
    conn.create_channel().await.expect("create AMQP channel")
}

pub fn unique_queue_name(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}

pub async fn declare_test_queue(channel: &Channel, name: &str) {
    channel
        .queue_declare(name, QueueDeclareOptions::default(), FieldTable::default())
        .await
        .expect("declare queue");
}
