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

use protocol::storage::codec::JournalEnginePacket;
use protocol::storage::journal_engine::RespHeader;

use crate::error::JournalClientError;

pub fn resp_header_error(
    resp_header: &Option<RespHeader>,
    request_pkg: JournalEnginePacket,
) -> Result<(), JournalClientError> {
    if let Some(header) = resp_header {
        if let Some(err) = header.error.clone() {
            return Err(JournalClientError::JournalEngineError(err.code, err.error));
        }
        return Ok(());
    }
    Err(JournalClientError::ReceivedPacketNotContainHeader(
        request_pkg.to_string(),
    ))
}
