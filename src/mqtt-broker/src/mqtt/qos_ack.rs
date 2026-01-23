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

use crate::core::cache::{QosAckPackageData, QosAckPackageType};
use crate::core::response::{
    response_packet_mqtt_distinct_by_reason, response_packet_mqtt_pubcomp_fail,
    response_packet_mqtt_pubcomp_success,
};

use common_base::tools::now_millis;
use protocol::mqtt::common::{
    DisconnectReasonCode, MqttPacket, PubAck, PubAckProperties, PubComp, PubCompProperties,
    PubCompReason, PubRec, PubRecProperties, PubRel, PubRelProperties,
};
use tracing::debug;

impl MqttService {
    pub async fn publish_ack(
        &self,
        connect_id: u64,
        pub_ack: &PubAck,
        _: &Option<PubAckProperties>,
    ) -> Option<MqttPacket> {
        if let Some(conn) = self.cache_manager.get_connection(connect_id) {
            let client_id = conn.client_id.clone();
            let pkid = pub_ack.pkid;
            if let Some(data) = self
                .cache_manager
                .pkid_metadata
                .get_ack_packet(&client_id, pkid)
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
                        conn.client_id,
                        pub_ack.pkid,
                        connect_id,
                        now_millis() - data.create_time
                    );
                }
            }
        }

        None
    }

    pub async fn publish_rec(
        &self,
        connect_id: u64,
        pub_rec: &PubRec,
        _: &Option<PubRecProperties>,
    ) -> Option<MqttPacket> {
        if let Some(conn) = self.cache_manager.get_connection(connect_id) {
            let client_id = conn.client_id;
            let pkid = pub_rec.pkid;
            if let Some(data) = self
                .cache_manager
                .pkid_metadata
                .get_ack_packet(&client_id, pkid)
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
                        client_id,
                        pub_rec.pkid,
                        connect_id,
                        now_millis() - data.create_time
                    );
                }
            }
        }

        None
    }

    pub async fn publish_comp(
        &self,
        connect_id: u64,
        pub_comp: &PubComp,
        _: &Option<PubCompProperties>,
    ) -> Option<MqttPacket> {
        if let Some(conn) = self.cache_manager.get_connection(connect_id) {
            let client_id = conn.client_id;
            let pkid = pub_comp.pkid;
            if let Some(data) = self
                .cache_manager
                .pkid_metadata
                .get_ack_packet(&client_id, pkid)
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
                        client_id,
                        pub_comp.pkid,
                        connect_id,
                        now_millis() - data.create_time
                    );
                }
            }
        }
        None
    }

    pub async fn publish_rel(
        &self,
        connect_id: u64,
        pub_rel: &PubRel,
        _: &Option<PubRelProperties>,
    ) -> MqttPacket {
        let connection = if let Some(se) = self.cache_manager.get_connection(connect_id) {
            se
        } else {
            return response_packet_mqtt_distinct_by_reason(
                &self.protocol,
                Some(DisconnectReasonCode::MaximumConnectTime),
                None,
            );
        };

        let client_id = connection.client_id.clone();
        if self
            .cache_manager
            .pkid_metadata
            .get_client_pkid(&client_id, pub_rel.pkid)
            .is_none()
        {
            return response_packet_mqtt_pubcomp_fail(
                &self.protocol,
                &connection,
                pub_rel.pkid,
                PubCompReason::PacketIdentifierNotFound,
                None,
            );
        }

        self.cache_manager
            .pkid_metadata
            .delete_client_pkid(&client_id, pub_rel.pkid);
        connection.recv_qos_message_decr();
        response_packet_mqtt_pubcomp_success(&self.protocol, pub_rel.pkid)
    }
}
