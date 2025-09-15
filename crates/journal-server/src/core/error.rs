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
use std::string::FromUtf8Error;

use common_base::error::common::CommonError;
use thiserror::Error;

use crate::segment::write::SegmentWriteData;

#[derive(Error, Debug)]
pub enum JournalServerError {
    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

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
    NotAvailableSegments(String),

    #[error("Shard {0} Not Active, is triggering the creation of an active Segment")]
    NotActiveSegment(String),

    #[error("segment {0} does not exist")]
    SegmentNotExist(String),

    #[error("Connection ID {0} information not found in cache.")]
    NotFoundConnectionInCache(u64),

    #[error("Segment {0} is currently in state {1} and is not allowed to write data")]
    SegmentStatusError(String, String),

    #[error("Segment {0} is already in the SealUp state and is not allowed to write")]
    SegmentAlreadySealUp(String),

    #[error("Current node is not the Leader of Segment {0}")]
    NotLeader(String),

    #[error("Segment file {0} does not exist, maybe it hasn't been initialized yet.")]
    SegmentFileNotExists(String),

    #[error("Data directory configuration for Segment {0} on node {1} cannot be found, please check that the metadata information is correct")]
    SegmentDataDirectoryNotFound(String, u64),

    #[error("Segment meta {0} does not exist, maybe it hasn't been initialized yet.")]
    SegmentMetaNotExists(String),

    #[error("Segment file meta {0} does not exist, maybe it hasn't been initialized yet.")]
    SegmentFileMetaNotExists(String),

    #[error("Timestamp {0} is less than the start timestamp {1} of Segment {2}, which should belong to the previous segment")]
    TimestampBelongToPreviousSegment(u64, i64, String),

    #[error("Timestamp {0} is greater than the start timestamp {1} of Segment {2}, which should belong to the next Segment")]
    TimestampBelongToNextSegment(u64, i64, String),

    #[error("Offset for timestamp {0} is not available in Segment {1}.")]
    NotAvailableOffsetByTimestamp(u64, String),

    #[error("Segment Offset is at the end and can no longer be written.")]
    SegmentOffsetAtTheEnd,
}

pub fn get_journal_server_code(e: &JournalServerError) -> String {
    match e {
        JournalServerError::CommonError(_) => "CommonError".to_string(),
        JournalServerError::BroadcastBoolSendError(_) => "BroadcastBoolSendError".to_string(),
        JournalServerError::MpscSegmentWriteDataSendError(_) => {
            "MpscSegmentWriteDataSendError".to_string()
        }
        JournalServerError::OneshotRecvError(_) => "OneshotRecvError".to_string(),
        JournalServerError::FromUtf8Error(_) => "FromUtf8Error".to_string(),
        JournalServerError::SegmentAlreadySealUp(_) => "SegmentAlreadySealUp".to_string(),
        JournalServerError::TokioTimeErrorElapsed(_) => "TokioTimeErrorElapsed".to_string(),
        JournalServerError::StdIoError(_) => "StdIoError".to_string(),
        JournalServerError::ProstDecodeError(_) => "ProstDecodeError".to_string(),
        JournalServerError::SerdeJsonError(_) => "SerdeJsonError".to_string(),
        JournalServerError::ParseIntError(_) => "ParseIntError".to_string(),
        JournalServerError::RequestBodyNotEmpty(_) => "RequestBodyNotEmpty".to_string(),
        JournalServerError::ShardNotExist(_) => "ShardNotExist".to_string(),
        JournalServerError::NotAvailableSegments(_) => "NotAvailableSegments".to_string(),
        JournalServerError::NotActiveSegment(_) => "NotActiveSegment".to_string(),
        JournalServerError::SegmentNotExist(_) => "SegmentNotExist".to_string(),
        JournalServerError::NotFoundConnectionInCache(_) => "NotFoundConnectionInCache".to_string(),
        JournalServerError::SegmentStatusError(_, _) => "SegmentStatusError".to_string(),
        JournalServerError::NotLeader(_) => "NotLeader".to_string(),
        JournalServerError::SegmentFileNotExists(_) => "SegmentFileNotExists".to_string(),
        JournalServerError::SegmentDataDirectoryNotFound(_, _) => {
            "SegmentDataDirectoryNotFound".to_string()
        }
        JournalServerError::SegmentMetaNotExists(_) => "SegmentMetaNotExists".to_string(),
        JournalServerError::SegmentFileMetaNotExists(_) => "SegmentFileMetaNotExists".to_string(),
        JournalServerError::TimestampBelongToPreviousSegment(_, _, _) => {
            "TimestampBelongToPreviousSegment".to_string()
        }
        JournalServerError::TimestampBelongToNextSegment(_, _, _) => {
            "TimestampBelongToNextSegment".to_string()
        }
        JournalServerError::NotAvailableOffsetByTimestamp(_, _) => {
            "NotAvailableOffsetByTimestamp".to_string()
        }
        JournalServerError::SegmentOffsetAtTheEnd => "SegmentOffsetAtTheEnd".to_string(),
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
