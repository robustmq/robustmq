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

use std::sync::Arc;

use super::MqttService;
use crate::core::{
    cache::{MQTTCacheManager, QosAckPackageData, QosAckPackageType},
    connection::is_request_problem_info,
};
use common_base::tools::now_millis;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    MqttPacket, MqttProtocol, PubAck, PubAckProperties, PubComp, PubCompProperties, PubCompReason,
    PubRec, PubRecProperties, PubRel, PubRelProperties,
};
use tracing::debug;

impl MqttService {
    pub async fn publish_ack(
        &self,
        connection: &MQTTConnection,
        pub_ack: &PubAck,
        _: &Option<PubAckProperties>,
    ) -> Option<MqttPacket> {
        let pkid = pub_ack.pkid;
        if let Some(data) = self
            .cache_manager
            .pkid_metadata
            .get_ack_packet(&connection.client_id, pkid)
        {
            if let Err(e) = data.sx.send(QosAckPackageData {
                ack_type: QosAckPackageType::PubAck,
                pkid: pub_ack.pkid,
            }) {
                debug!(
                        "send puback to channel fail, error message:{}, send data time: {}, recv ack time:{}, client_id: {}, pkid: {}, connect_id:{}, diff:{}ms",
                        e,
                        data.create_time,
                        now_millis(),
                        connection.client_id,
                        pub_ack.pkid,
                        connection.connect_id,
                        now_millis() - data.create_time
                    );
            }
        }

        None
    }

    pub async fn publish_rec(
        &self,
        connection: &MQTTConnection,
        pub_rec: &PubRec,
        _: &Option<PubRecProperties>,
    ) -> Option<MqttPacket> {
        let pkid = pub_rec.pkid;
        if let Some(data) = self
            .cache_manager
            .pkid_metadata
            .get_ack_packet(&connection.client_id, pkid)
        {
            if let Err(e) = data.sx.send(QosAckPackageData {
                ack_type: QosAckPackageType::PubRec,
                pkid: pub_rec.pkid,
            }) {
                debug!(
                        "send pubrec to channel fail, error message:{}, send data time: {}, recv rec time:{}, client_id: {}, pkid: {}, connect_id:{}, diff:{}ms",
                        e,
                        data.create_time,
                        now_millis(),
                        connection.client_id,
                        pub_rec.pkid,
                        connection.connect_id,
                        now_millis() - data.create_time
                    );
            }
        }

        None
    }

    pub async fn publish_comp(
        &self,
        connection: &MQTTConnection,
        pub_comp: &PubComp,
        _: &Option<PubCompProperties>,
    ) -> Option<MqttPacket> {
        let pkid = pub_comp.pkid;
        if let Some(data) = self
            .cache_manager
            .pkid_metadata
            .get_ack_packet(&connection.client_id, pkid)
        {
            if let Err(e) = data.sx.send(QosAckPackageData {
                ack_type: QosAckPackageType::PubComp,
                pkid: pub_comp.pkid,
            }) {
                debug!(
                        "send pubcomp to channel fail, error message:{}, send data time: {}, recv comp time:{}, client_id: {}, pkid: {}, connect_id:{}, diff:{}ms",
                        e,
                        data.create_time,
                        now_millis(),
                        connection.client_id,
                        pub_comp.pkid,
                        connection.connect_id,
                        now_millis() - data.create_time
                    );
            }
        }

        None
    }

    pub async fn publish_rel(
        &self,
        connection: &MQTTConnection,
        pub_rel: &PubRel,
        _: &Option<PubRelProperties>,
    ) -> MqttPacket {
        let client_id = connection.client_id.clone();
        if self
            .cache_manager
            .pkid_metadata
            .get_client_pkid(&client_id, pub_rel.pkid)
            .is_none()
        {
            return response_packet_mqtt_pub_comp_fail(
                &self.cache_manager,
                connection.connect_id,
                &self.protocol,
                pub_rel.pkid,
                PubCompReason::PacketIdentifierNotFound,
                Some("".to_string()),
            );
        }

        self.cache_manager
            .pkid_metadata
            .delete_client_pkid(&client_id, pub_rel.pkid);
        connection.recv_qos_message_decr();
        response_packet_mqtt_pub_comp_success(&self.protocol, pub_rel.pkid)
    }
}

fn response_packet_mqtt_pub_comp_success(protocol: &MqttProtocol, pkid: u16) -> MqttPacket {
    let comp = PubComp {
        pkid,
        reason: Some(PubCompReason::Success),
    };

    if !protocol.is_mqtt5() {
        return MqttPacket::PubComp(comp, None);
    }

    MqttPacket::PubComp(comp, Some(PubCompProperties::default()))
}

fn response_packet_mqtt_pub_comp_fail(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
    pkid: u16,
    reason: PubCompReason,
    reason_string: Option<String>,
) -> MqttPacket {
    debug!(
        connect_id = connect_id,
        pkid = pkid,
        protocol = ?protocol,
        reason = ?reason,
        reason_string = ?reason_string,
        "Building publish complete failure packet"
    );

    let pub_comp = PubComp {
        pkid,
        reason: Some(reason),
    };

    if !protocol.is_mqtt5() {
        return MqttPacket::PubComp(pub_comp, None);
    }

    let mut properties = PubCompProperties::default();
    if is_request_problem_info(cache_manager, connect_id) {
        properties.reason_string = reason_string;
    }
    MqttPacket::PubComp(pub_comp, Some(properties))
}
