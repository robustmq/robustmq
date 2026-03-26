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

use crate::mqtt::MqttVersion;
use rumqttc::v5::mqttbytes::v5::ConnectReturnCode as V5ConnectReturnCode;
use rumqttc::v5::{
    AsyncClient as AsyncClientV5, EventLoop as EventLoopV5, MqttOptions as MqttOptionsV5,
};
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use std::time::Duration;

pub enum ClientHandle {
    V4(AsyncClient, Box<EventLoop>),
    V5(AsyncClientV5, Box<EventLoopV5>),
}

pub fn build_client(
    client_id: &str,
    host: &str,
    port: u16,
    username: &Option<String>,
    password: &Option<String>,
    version: MqttVersion,
) -> ClientHandle {
    match version {
        MqttVersion::V5 => {
            let mut opts = MqttOptionsV5::new(client_id, host, port);
            opts.set_keep_alive(Duration::from_secs(60));
            if let (Some(u), Some(p)) = (username, password) {
                opts.set_credentials(u.clone(), p.clone());
            }
            let (client, event_loop) = AsyncClientV5::new(opts, 1000);
            ClientHandle::V5(client, Box::new(event_loop))
        }
        MqttVersion::V4 => {
            let mut opts = MqttOptions::new(client_id, host, port);
            opts.set_keep_alive(Duration::from_secs(60));
            if let (Some(u), Some(p)) = (username, password) {
                opts.set_credentials(u.clone(), p.clone());
            }
            let (client, event_loop) = AsyncClient::new(opts, 1000);
            ClientHandle::V4(client, Box::new(event_loop))
        }
    }
}

pub async fn wait_connack_v4(event_loop: &mut EventLoop, timeout_ms: u64) -> Result<(), String> {
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

pub async fn wait_connack_v5(event_loop: &mut EventLoopV5, timeout_ms: u64) -> Result<(), String> {
    use rumqttc::v5::{Event as EventV5, Incoming as IncomingV5};
    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err("connect timeout".to_string());
        }
        match tokio::time::timeout(Duration::from_millis(200), event_loop.poll()).await {
            Ok(Ok(EventV5::Incoming(IncomingV5::ConnAck(ack)))) => {
                if ack.code == V5ConnectReturnCode::Success {
                    return Ok(());
                } else {
                    return Err(format!("connack rejected: {:?}", ack.code));
                }
            }
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
