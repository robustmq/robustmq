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
use common_base::tools::now_millis;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    MqttPacket, PubAck, PubAckProperties, PubComp, PubCompProperties, PubRec, PubRecProperties,
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
            .qos_data
            .get_publish_to_client_qos_ack_data(&connection.client_id, pkid)
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
            .qos_data
            .get_publish_to_client_qos_ack_data(&connection.client_id, pkid)
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
            .qos_data
            .get_publish_to_client_qos_ack_data(&connection.client_id, pkid)
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
}
