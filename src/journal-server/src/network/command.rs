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

use super::{response::build_produce_resp, services::Services};

use log::error;
use protocol::journal_server::codec::StorageEnginePacket;

pub struct Command {
    packet: StorageEnginePacket,
    services: Services,
}

impl Command {
    pub fn new(packet: StorageEnginePacket) -> Self {
        let services = Services::new();
        return Command { packet, services };
    }

    pub fn apply(&self) -> StorageEnginePacket {
        match self.packet.clone() {
            StorageEnginePacket::ProduceReq(data) => {
                self.services.produce(data);
            }
            StorageEnginePacket::FetchReq(data) => {
                self.services.fetch(data);
            }
            _ => {
                error!(
                    "server received an unrecognized request, request info: {:?}",
                    self.packet
                );
            }
        }
        return build_produce_resp();
    }
}
