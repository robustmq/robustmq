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

use kafka_protocol::messages::{
    CreateDelegationTokenRequest, DescribeDelegationTokenRequest, ExpireDelegationTokenRequest,
    RenewDelegationTokenRequest,
};
use protocol::kafka::packet::KafkaPacket;

pub fn process_create_delegation_token(_req: &CreateDelegationTokenRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_renew_delegation_token(_req: &RenewDelegationTokenRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_expire_delegation_token(_req: &ExpireDelegationTokenRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_delegation_token(
    _req: &DescribeDelegationTokenRequest,
) -> Option<KafkaPacket> {
    None
}
