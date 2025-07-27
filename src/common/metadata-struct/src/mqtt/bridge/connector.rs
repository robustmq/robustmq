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

use serde::{Deserialize, Serialize};

use super::{connector_type::ConnectorType, status::MQTTStatus};
use protocol::broker_mqtt::broker_mqtt_admin::{
    ConnectorRaw, ConnectorType as ProtoConnectorType, MqttStatus as ProtoMQTTStatus,
};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MQTTConnector {
    pub cluster_name: String,
    pub connector_name: String,
    pub connector_type: ConnectorType,
    pub config: String,
    pub topic_id: String,
    pub status: MQTTStatus,
    pub broker_id: Option<u64>,
    pub create_time: u64,
    pub update_time: u64,
}

impl MQTTConnector {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    pub fn decode(data: &[u8]) -> Self {
        serde_json::from_slice(data).unwrap()
    }
}

impl From<ConnectorType> for ProtoConnectorType {
    fn from(type_enum: ConnectorType) -> Self {
        match type_enum {
            ConnectorType::Kafka => ProtoConnectorType::Kafka,
            ConnectorType::LocalFile => ProtoConnectorType::File,
        }
    }
}

impl From<MQTTConnector> for ConnectorRaw {
    fn from(connector: MQTTConnector) -> Self {
        let connector_type: ProtoConnectorType = connector.connector_type.into();
        let connector_status: ProtoMQTTStatus = connector.status.into();
        Self {
            cluster_name: connector.cluster_name,
            connector_name: connector.connector_name,
            connector_type: connector_type as i32,
            config: connector.config,
            topic_id: connector.topic_id,
            status: connector_status as i32,
            broker_id: connector.broker_id,
            create_time: connector.create_time,
            update_time: connector.update_time,
        }
    }
}
