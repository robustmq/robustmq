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

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonReply {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ClusterType {
    PlacementCenter = 0,
    JournalServer = 1,
    MqttBrokerServer = 2,
    AmqpBrokerServer = 3,
}
impl ClusterType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ClusterType::PlacementCenter => "PlacementCenter",
            ClusterType::JournalServer => "JournalServer",
            ClusterType::MqttBrokerServer => "MQTTBrokerServer",
            ClusterType::AmqpBrokerServer => "AMQPBrokerServer",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PlacementCenter" => Some(Self::PlacementCenter),
            "JournalServer" => Some(Self::JournalServer),
            "MQTTBrokerServer" => Some(Self::MqttBrokerServer),
            "AMQPBrokerServer" => Some(Self::AmqpBrokerServer),
            _ => None,
        }
    }
}
