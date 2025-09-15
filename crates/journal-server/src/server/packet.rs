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

use std::net::SocketAddr;

use protocol::journal::codec::JournalEnginePacket;

#[derive(Clone, Debug)]
pub struct RequestPackage {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub packet: JournalEnginePacket,
}

impl RequestPackage {
    pub fn new(connection_id: u64, addr: SocketAddr, packet: JournalEnginePacket) -> Self {
        Self {
            connection_id,
            addr,
            packet,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResponsePackage {
    pub connection_id: u64,
    pub packet: JournalEnginePacket,
}

impl ResponsePackage {
    pub fn new(connection_id: u64, packet: JournalEnginePacket) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}
