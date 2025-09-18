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

use bytes::Bytes;
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::mqtt::common::{Connect, ConnectProperties, LastWill, Login, MqttPacket};
use uuid::Uuid;

pub fn build_test_mqtt4_connect_packet_wrapper() -> MqttPacketWrapper {
    MqttPacketWrapper {
        protocol_version: 4,
        packet: build_test_mqtt4_connect_packet(),
    }
}
pub fn build_test_mqtt4_connect_packet() -> MqttPacket {
    let random_client_id = create_random_client_id();
    let connect = Connect {
        keep_alive: 0,
        client_id: random_client_id,
        clean_session: false,
    };
    MqttPacket::Connect(4, connect, None, None, None, None)
}

fn create_random_client_id() -> String {
    let uuid = Uuid::new_v4();
    let client_id = format!("test_client_id_{uuid}");
    client_id
}

/// Build the connect content package for the mqtt4 protocol
pub fn build_mqtt4_connect_packet() -> MqttPacket {
    let client_id = String::from("test_client_id");
    let login = Some(Login {
        username: "lobo".to_string(),
        password: "123456".to_string(),
    });
    let lastwill = Some(LastWill {
        topic: Bytes::from("topic1"),
        message: Bytes::from("connection content"),
        qos: protocol::mqtt::common::QoS::AtLeastOnce,
        retain: true,
    });

    let connect: Connect = Connect {
        keep_alive: 30u16, // 30 seconds
        client_id,
        clean_session: true,
    };
    MqttPacket::Connect(4, connect, None, lastwill, None, login)
}

/// Build the connect content package for the mqtt5 protocol
pub fn build_mqtt5_pg_connect() -> MqttPacket {
    let client_id = String::from("test_client_id");
    let login = Some(Login {
        username: "lobo".to_string(),
        password: "123456".to_string(),
    });
    let lastwill = Some(LastWill {
        topic: Bytes::from("topic1"),
        message: Bytes::from("connection content"),
        qos: protocol::mqtt::common::QoS::AtLeastOnce,
        retain: true,
    });

    let connect: Connect = Connect {
        keep_alive: 30u16, // 30 seconds
        client_id,
        clean_session: true,
    };

    let properties = ConnectProperties {
        session_expiry_interval: Some(30),
        ..Default::default()
    };
    MqttPacket::Connect(5, connect, Some(properties), lastwill, None, login)
}

pub fn build_mqtt5_pg_connect_wrapper() -> MqttPacketWrapper {
    MqttPacketWrapper {
        protocol_version: 5,
        packet: build_mqtt5_pg_connect(),
    }
}
