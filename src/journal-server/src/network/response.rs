// Copyright 2023 RobustMQ Team
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
    codec::StorageEnginePacket,
    generate::protocol::{
        header::{ApiKey, ApiType, ApiVersion, Header, RequestCommon, ResponseCommon},
        produce::{ProduceResp, ProduceRespBody},
    },
};

pub fn build_produce_resp() -> StorageEnginePacket {
    let header = build_req_header(33, "client-id-fsfsdff".to_string());
    let body = ProduceRespBody {};
    let req = ProduceResp {
        header: Some(header),
        body: Some(body),
    };
    return StorageEnginePacket::ProduceResp(req);
}

fn build_req_header(correlation_id: u32, client_id: String) -> Header {
    return Header {
        api_key: ApiKey::Produce.into(),
        api_type: ApiType::Response.into(),
        api_version: ApiVersion::V0.into(),
        request: Some(RequestCommon {
            correlation_id: correlation_id,
            client_id: client_id,
        }),
        response: None,
    };
}
