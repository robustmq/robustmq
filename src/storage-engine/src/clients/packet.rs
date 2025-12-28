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

use metadata_struct::storage::adapter_read_config::AdapterWriteRespRow;
use protocol::storage::protocol::{
    ApiKey, ReadReq, ReadReqBody, ReadReqMessage, ReadResp, ReadRespBody, ReqHeader, RespHeader,
    StorageEngineNetworkError, WriteReq, WriteReqBody, WriteResp, WriteRespBody, WriteRespMessage,
};

use crate::core::error::StorageEngineError;

pub fn build_write_req(shard_name: String, segment: u32, messages: Vec<Vec<u8>>) -> WriteReq {
    WriteReq {
        header: ReqHeader {
            api_key: ApiKey::Write,
        },
        body: WriteReqBody {
            shard_name,
            segment,
            messages,
        },
    }
}

pub fn build_write_resp(
    messages: Vec<WriteRespMessage>,
    error: Option<StorageEngineNetworkError>,
) -> WriteResp {
    WriteResp {
        header: RespHeader {
            api_key: ApiKey::Write,
            error,
        },
        body: WriteRespBody { status: messages },
    }
}

pub fn build_read_req(messages: Vec<ReadReqMessage>) -> ReadReq {
    ReadReq {
        header: ReqHeader {
            api_key: ApiKey::Read,
        },
        body: ReadReqBody { messages },
    }
}

pub fn build_read_resp(
    messages: Vec<Vec<u8>>,
    error: Option<StorageEngineNetworkError>,
) -> ReadResp {
    ReadResp {
        header: RespHeader {
            api_key: ApiKey::Read,
            error,
        },
        body: ReadRespBody { messages },
    }
}

// todo: In the future, there may be situations where some records are successfully written while others fail.
pub fn write_resp_parse(resp: &WriteResp) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
    if let Some(err) = &resp.header.error {
        return Err(StorageEngineError::CommonErrorStr(err.to_str()));
    }

    let mut results = Vec::new();
    for msg in &resp.body.status {
        for raw in &msg.messages {
            if let Some(err) = raw.error.clone() {
                results.push(AdapterWriteRespRow {
                    pkid: raw.pkid,
                    error: Some(err.to_str()),
                    ..Default::default()
                });
            } else {
                results.push(AdapterWriteRespRow {
                    pkid: raw.pkid,
                    offset: raw.offset,
                    ..Default::default()
                });
            }
        }
    }
    Ok(results)
}
