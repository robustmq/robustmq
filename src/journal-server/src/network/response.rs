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

use protocol::journal_server::{
    codec::JournalEnginePacket,
    generate::protocol::read::{ApiKey, ApiVersion, RespHeader, WriteResp, WriteRespBody},
};

pub fn build_produce_resp() -> JournalEnginePacket {
    let header = build_resp_header();
    let body = WriteRespBody {};
    let req = WriteResp {
        header: Some(header),
        body: Some(body),
    };
    JournalEnginePacket::WriteResp(req)
}

fn build_resp_header() -> RespHeader {
    RespHeader {
        api_key: ApiKey::Write.into(),
        api_version: ApiVersion::V0.into(),
    }
}
