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

use std::num::ParseIntError;

use common_base::error::common::CommonError;
use thiserror::Error;

use super::write::SegmentWriteData;

#[derive(Error, Debug)]
pub enum JournalServerError {
    #[error("Directory {0} No rocksdb instance available")]
    NoRocksdbInstanceAvailable(String),

    #[error("{0}")]
    CommonError(#[from] CommonError),

    #[error("{0}")]
    StdIoError(#[from] std::io::Error),

    #[error("{0}")]
    BroadcastBoolSendError(#[from] tokio::sync::broadcast::error::SendError<bool>),

    #[error("{0}")]
    MpscSegmentWriteDataSendError(#[from] tokio::sync::mpsc::error::SendError<SegmentWriteData>),

    #[error("{0}")]
    OneshotRecvError(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("{0}")]
    ProstDecodeError(#[from] prost::DecodeError),

    #[error("{0}")]
    TokioTimeErrorElapsed(#[from] tokio::time::error::Elapsed),

    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("{0} request body cannot be empty")]
    RequestBodyNotEmpty(String),

    #[error("Shard {0} does not exist")]
    ShardNotExist(String),

    #[error("No Segment is available for Shard {0}")]
    NotAvailableSegmets(String),

    #[error("Shard {0} Not Active, is triggering the creation of an active Segment")]
    NotActiveSegmet(String),

    #[error("Shard {0},segment {1} does not exist")]
    SegmentNotExist(String, u32),

    #[error("Connection ID {0} information not found in cache.")]
    NotFoundConnectionInCache(u64),

    #[error("Segment {0} is currently in state {1} and is not allowed to write data")]
    SegmentStatusError(String, String),

    #[error("Segment {0} is already in the SealUp state and is not allowed to write")]
    SegmentAlreadySealUp(String),

    #[error("Current node is not the Leader of Segment {1} in the shard {0}")]
    NotLeader(String, u32),

    #[error("Segment file {0} already exists. We can't create Segment file again and again.")]
    SegmentFileAlreadyExists(String),

    #[error("Segment file {0} does not exist, maybe it hasn't been initialized yet.")]
    SegmentFileNotExists(String),

    #[error("Data directory configuration for Segment {0} on node {1} cannot be found, please check that the metadata information is correct")]
    SegmentDataDirectoryNotFound(String, u64),

    #[error("Segment meta {0} does not exist, maybe it hasn't been initialized yet.")]
    SegmentMetaNotExists(String),
}

pub fn get_journal_server_code(e: &JournalServerError) -> String {
    match e {
        JournalServerError::NoRocksdbInstanceAvailable(_) => {
            "NoRocksdbInstanceAvailable".to_string()
        }
        JournalServerError::CommonError(_) => "CommonError".to_string(),
        JournalServerError::BroadcastBoolSendError(_) => "BroadcastBoolSendError".to_string(),
        JournalServerError::MpscSegmentWriteDataSendError(_) => {
            "MpscSegmentWriteDataSendError".to_string()
        }
        JournalServerError::OneshotRecvError(_) => "OneshotRecvError".to_string(),
        JournalServerError::SegmentAlreadySealUp(_) => "SegmentAlreadySealUp".to_string(),
        JournalServerError::TokioTimeErrorElapsed(_) => "TokioTimeErrorElapsed".to_string(),
        JournalServerError::StdIoError(_) => "StdIoError".to_string(),
        JournalServerError::ProstDecodeError(_) => "ProstDecodeError".to_string(),
        JournalServerError::SerdeJsonError(_) => "SerdeJsonError".to_string(),
        JournalServerError::ParseIntError(_) => "ParseIntError".to_string(),
        JournalServerError::RequestBodyNotEmpty(_) => "RequestBodyNotEmpty".to_string(),
        JournalServerError::ShardNotExist(_) => "ShardNotExist".to_string(),
        JournalServerError::NotAvailableSegmets(_) => "NotAvailableSegmets".to_string(),
        JournalServerError::NotActiveSegmet(_) => "NotActiveSegmet".to_string(),
        JournalServerError::SegmentNotExist(_, _) => "SegmentNotExist".to_string(),
        JournalServerError::NotFoundConnectionInCache(_) => "NotFoundConnectionInCache".to_string(),
        JournalServerError::SegmentStatusError(_, _) => "SegmentStatusError".to_string(),
        JournalServerError::NotLeader(_, _) => "NotLeader".to_string(),
        JournalServerError::SegmentFileAlreadyExists(_) => "SegmentFileAlreadyExists".to_string(),
        JournalServerError::SegmentFileNotExists(_) => "SegmentFileNotExists".to_string(),
        JournalServerError::SegmentDataDirectoryNotFound(_, _) => {
            "SegmentDataDirectoryNotFound".to_string()
        }
        JournalServerError::SegmentMetaNotExists(_) => "SegmentMetaNotExists".to_string(),
    }
}
#[cfg(test)]
mod tests {
    use super::{get_journal_server_code, JournalServerError};

    #[tokio::test]
    async fn get_journal_server_code_test() {
        let e = JournalServerError::SegmentMetaNotExists("xx".to_string());
        assert_eq!(
            get_journal_server_code(&e),
            "SegmentMetaNotExists".to_string()
        );
    }
}
