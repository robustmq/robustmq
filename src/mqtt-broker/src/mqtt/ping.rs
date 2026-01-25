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

use super::MqttService;
use crate::core::cache::ConnectionLiveTime;
use common_base::tools::now_second;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{MqttPacket, PingReq, PingResp};

pub fn response_packet_mqtt_ping_resp() -> MqttPacket {
    MqttPacket::PingResp(PingResp {})
}

impl MqttService {
    pub async fn ping(&self, connection: &MQTTConnection, _: &PingReq) -> MqttPacket {
        let live_time = ConnectionLiveTime {
            protocol: self.protocol.clone(),
            keep_live: connection.keep_alive,
            heartbeat: now_second(),
        };
        self.cache_manager
            .report_heartbeat(connection.client_id.clone(), live_time);
        self.connection_manager
            .report_heartbeat(connection.connect_id, now_second());
        response_packet_mqtt_ping_resp()
    }
}
