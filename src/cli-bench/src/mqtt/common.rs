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

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use std::time::Duration;

pub fn build_client(
    client_id: &str,
    host: &str,
    port: u16,
    username: &Option<String>,
    password: &Option<String>,
) -> (AsyncClient, EventLoop) {
    let mut opts = MqttOptions::new(client_id, host, port);
    opts.set_keep_alive(Duration::from_secs(30));
    if let (Some(u), Some(p)) = (username, password) {
        opts.set_credentials(u.clone(), p.clone());
    }
    AsyncClient::new(opts, 1000)
}

pub async fn wait_connack(event_loop: &mut EventLoop, timeout_ms: u64) -> Result<(), String> {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err("connect timeout".to_string());
        }
        match tokio::time::timeout(Duration::from_millis(200), event_loop.poll()).await {
            Ok(Ok(Event::Incoming(Incoming::ConnAck(_)))) => return Ok(()),
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(e.to_string()),
            Err(_) => {}
        }
    }
}

pub fn qos_from_u8(qos: u8) -> QoS {
    match qos {
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtMostOnce,
    }
}
