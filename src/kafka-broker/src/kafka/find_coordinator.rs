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

use kafka_protocol::messages::find_coordinator_response::Coordinator;
use kafka_protocol::messages::{FindCoordinatorRequest, FindCoordinatorResponse};
use protocol::kafka::packet::KafkaPacket;

pub fn process_find_coordinator(_req: &FindCoordinatorRequest) -> Option<KafkaPacket> {
    let resp = FindCoordinatorResponse::default()
        .with_error_code(0)
        .with_node_id(0.into())
        .with_host("127.0.0.1".into())
        .with_port(9092)
        .with_coordinators(vec![Coordinator::default()
            .with_error_code(0)
            .with_node_id(0.into())
            .with_host("127.0.0.1".into())
            .with_port(9092)]);

    Some(KafkaPacket::FindCoordinatorResponse(resp))
}
