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

use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::connack::ConnAckFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::connect::ConnectFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::disconnect::DisconnectFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::pingreq::PingReqFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::pingresp::PingRespFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::puback::PubAckFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::pubcomp::PubCompFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::publish::PublishFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::pubrec::PubRecFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::pubrel::PubRelFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::suback::SubAckFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::subscribe::SubscribeFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::unsuback::UnsubAckFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::fixed_header::unsubscribe::UnsubscribeFixedHeader;
use crate::mqtt::adapter::mqtt_3_1_1::payload::connect::ConnectPayload;
use crate::mqtt::adapter::mqtt_3_1_1::payload::publish::PublishPayload;
use crate::mqtt::adapter::mqtt_3_1_1::payload::suback::SubAckPayload;
use crate::mqtt::adapter::mqtt_3_1_1::payload::subscribe::SubscribePayload;
use crate::mqtt::adapter::mqtt_3_1_1::payload::unsubscribe::UnsubscribePayload;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::connack::ConnAckVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::connect::ConnectVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::puback::PubAckVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::pubcomp::PubCompVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::publish::PublishVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::pubrec::PubRecVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::pubrel::PubRelVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::suback::SubAckVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::subscribe::SubscribeVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::unsuback::UnsubAckVariableHeader;
use crate::mqtt::adapter::mqtt_3_1_1::variable_header::unsubscribe::UnsubscribeVariableHeader;
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Connect(ConnectFixedHeader, ConnectVariableHeader, ConnectPayload),
    ConnAck(ConnAckFixedHeader, ConnAckVariableHeader),
    Publish(PublishFixedHeader, PublishVariableHeader, PublishPayload),
    PubAck(PubAckFixedHeader, PubAckVariableHeader),
    PubRec(PubRecFixedHeader, PubRecVariableHeader),
    PubRel(PubRelFixedHeader, PubRelVariableHeader),
    PubComp(PubCompFixedHeader, PubCompVariableHeader),
    Subscribe(
        SubscribeFixedHeader,
        SubscribeVariableHeader,
        SubscribePayload,
    ),
    SubAck(SubAckFixedHeader, SubAckVariableHeader, SubAckPayload),
    Unsubscribe(
        UnsubscribeFixedHeader,
        UnsubscribeVariableHeader,
        UnsubscribePayload,
    ),
    UnsubAck(UnsubAckFixedHeader, UnsubAckVariableHeader),
    PingReq(PingReqFixedHeader),
    PingResp(PingRespFixedHeader),
    Disconnect(DisconnectFixedHeader),
}

#[allow(dead_code)]
impl Packet {
    pub fn new_connect(variable_header: ConnectVariableHeader, payload: ConnectPayload) -> Self {
        Packet::Connect(ConnectFixedHeader::new(), variable_header, payload)
    }

    pub fn new_connack(variable_header: ConnAckVariableHeader) -> Self {
        Packet::ConnAck(ConnAckFixedHeader::new(), variable_header)
    }

    pub fn new_publish(
        fixed_header: PublishFixedHeader,
        variable_header: PublishVariableHeader,
        payload: PublishPayload,
    ) -> Self {
        Packet::Publish(fixed_header, variable_header, payload)
    }

    pub fn new_puback(variable_header: PubAckVariableHeader) -> Self {
        Packet::PubAck(PubAckFixedHeader::new(), variable_header)
    }

    pub fn new_pubrec(variable_header: PubRecVariableHeader) -> Self {
        Packet::PubRec(PubRecFixedHeader::new(), variable_header)
    }
    pub fn new_pubrel(variable_header: PubRelVariableHeader) -> Self {
        Packet::PubRel(PubRelFixedHeader::new(), variable_header)
    }

    pub fn new_pubcomp(variable_header: PubCompVariableHeader) -> Self {
        Packet::PubComp(PubCompFixedHeader::new(), variable_header)
    }

    pub fn new_subscribe(
        variable_header: SubscribeVariableHeader,
        payload: SubscribePayload,
    ) -> Self {
        Packet::Subscribe(SubscribeFixedHeader::new(), variable_header, payload)
    }

    pub fn new_suback(variable_header: SubAckVariableHeader, payload: SubAckPayload) -> Self {
        Packet::SubAck(SubAckFixedHeader::new(), variable_header, payload)
    }
    pub fn new_unsubscribe(
        variable_header: UnsubscribeVariableHeader,
        payload: UnsubscribePayload,
    ) -> Self {
        Packet::Unsubscribe(UnsubscribeFixedHeader::new(), variable_header, payload)
    }
    pub fn new_unsuback(variable_header: UnsubAckVariableHeader) -> Self {
        Packet::UnsubAck(UnsubAckFixedHeader::new(), variable_header)
    }
    pub fn new_pingreq() -> Self {
        Packet::PingReq(PingReqFixedHeader::new())
    }
    pub fn new_pingresp() -> Self {
        Packet::PingResp(PingRespFixedHeader::new())
    }
    pub fn new_disconnect() -> Self {
        Packet::Disconnect(DisconnectFixedHeader::new())
    }
}
