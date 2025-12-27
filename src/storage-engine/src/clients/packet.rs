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

use protocol::storage::protocol::{
    ApiKey, ReadReq, ReadReqBody, ReadReqMessage, ReadResp, ReadRespBody, ReadRespSegmentMessage,
    ReqHeader, RespHeader, StorageEngineNetworkError, WriteReq, WriteReqBody, WriteReqMessages,
    WriteResp, WriteRespBody, WriteRespMessage,
};

pub fn build_write_req(
    shard_name: String,
    segment: u32,
    messages: Vec<WriteReqMessages>,
) -> WriteReq {
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
    messages: Vec<ReadRespSegmentMessage>,
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
