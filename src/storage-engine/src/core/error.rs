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

use common_base::error::common::CommonError;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use thiserror::Error;

use crate::segment::write::WriteChannelData;

#[derive(Error, Debug)]
pub enum StorageEngineError {
    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("{0}")]
    CommonError(Box<CommonError>),

    #[error("{0}")]
    StdIoError(#[from] std::io::Error),

    #[error("{0}")]
    BroadcastBoolSendError(#[from] tokio::sync::broadcast::error::SendError<bool>),

    #[error("{0}")]
    MpscSegmentWriteDataSendError(#[from] tokio::sync::mpsc::error::SendError<WriteChannelData>),

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

    #[error("No available IO thread available.")]
    NoAvailableIoThread,

    #[error("Error occurred while reading Segment File {0} data")]
    ReadSegmentFileError(String),

    #[error("No Offset information for the Shard {0}stored locally. Please check the index data.")]
    NoOffsetInformation(String),

    #[error("{0}")]
    CommonErrorStr(String),

    #[error("Failed to send request to node {0} with error message :{1}")]
    SendRequestError(i64, String),

    #[error("request is sent to node {0} and the received return packet is empty")]
    ReceivedPacketIsEmpty(i64),

    #[error("Send request to node {0}, received wrong packet, error message :{1}")]
    ReceivedPacketError(i64, String),

    #[error("Sending a request to node {0}, obtaining a connection failed, and the connection is occupied for a long time.")]
    ConnectionIsOccupied(i64),

    #[error("Sending a request to node {0} failed to get a connection, possibly to create a connection.")]
    NoAvailableConn(i64),
}

impl From<CommonError> for StorageEngineError {
    fn from(error: CommonError) -> Self {
        StorageEngineError::CommonError(Box::new(error))
    }
}

pub fn get_journal_server_code(e: &StorageEngineError) -> String {
    match e {
        StorageEngineError::CommonError(_) => "CommonError".to_string(),
        StorageEngineError::BroadcastBoolSendError(_) => "BroadcastBoolSendError".to_string(),
        StorageEngineError::MpscSegmentWriteDataSendError(_) => {
            "MpscSegmentWriteDataSendError".to_string()
        }
        StorageEngineError::OneshotRecvError(_) => "OneshotRecvError".to_string(),
        StorageEngineError::FromUtf8Error(_) => "FromUtf8Error".to_string(),
        StorageEngineError::SegmentAlreadySealUp(_) => "SegmentAlreadySealUp".to_string(),
        StorageEngineError::TokioTimeErrorElapsed(_) => "TokioTimeErrorElapsed".to_string(),
        StorageEngineError::StdIoError(_) => "StdIoError".to_string(),
        StorageEngineError::ProstDecodeError(_) => "ProstDecodeError".to_string(),
        StorageEngineError::SerdeJsonError(_) => "SerdeJsonError".to_string(),
        StorageEngineError::ParseIntError(_) => "ParseIntError".to_string(),
        StorageEngineError::RequestBodyNotEmpty(_) => "RequestBodyNotEmpty".to_string(),
        StorageEngineError::ShardNotExist(_) => "ShardNotExist".to_string(),
        StorageEngineError::NotAvailableSegments(_) => "NotAvailableSegments".to_string(),
        StorageEngineError::NotActiveSegment(_) => "NotActiveSegment".to_string(),
        StorageEngineError::SegmentNotExist(_) => "SegmentNotExist".to_string(),
        StorageEngineError::NotFoundConnectionInCache(_) => "NotFoundConnectionInCache".to_string(),
        StorageEngineError::SegmentStatusError(_, _) => "SegmentStatusError".to_string(),
        StorageEngineError::NotLeader(_) => "NotLeader".to_string(),
        StorageEngineError::SegmentFileNotExists(_) => "SegmentFileNotExists".to_string(),
        StorageEngineError::SegmentDataDirectoryNotFound(_, _) => {
            "SegmentDataDirectoryNotFound".to_string()
        }
        StorageEngineError::SegmentMetaNotExists(_) => "SegmentMetaNotExists".to_string(),
        StorageEngineError::SegmentFileMetaNotExists(_) => "SegmentFileMetaNotExists".to_string(),
        StorageEngineError::TimestampBelongToPreviousSegment(_, _, _) => {
            "TimestampBelongToPreviousSegment".to_string()
        }
        StorageEngineError::TimestampBelongToNextSegment(_, _, _) => {
            "TimestampBelongToNextSegment".to_string()
        }
        StorageEngineError::NotAvailableOffsetByTimestamp(_, _) => {
            "NotAvailableOffsetByTimestamp".to_string()
        }
        StorageEngineError::SegmentOffsetAtTheEnd => "SegmentOffsetAtTheEnd".to_string(),
        StorageEngineError::NoAvailableIoThread => "NoAvailableIoThread".to_string(),
        StorageEngineError::ReadSegmentFileError(_) => "ReadSegmentFileError".to_string(),
        StorageEngineError::NoOffsetInformation(_) => "NoOffsetInformation".to_string(),
        StorageEngineError::CommonErrorStr(_) => "NoOffsetInformation".to_string(),
        StorageEngineError::SendRequestError(_, _) => "SendRequestError".to_string(),
        StorageEngineError::ReceivedPacketIsEmpty(_) => "ReceivedPacketIsEmpty".to_string(),
        StorageEngineError::ReceivedPacketError(_, _) => "ReceivedPacketError".to_string(),
        StorageEngineError::ConnectionIsOccupied(_) => "ConnectionIsOccupied".to_string(),
        StorageEngineError::NoAvailableConn(_) => "NoAvailableConn".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{get_journal_server_code, StorageEngineError};

    #[tokio::test]
    async fn get_journal_server_code_test() {
        let e = StorageEngineError::SegmentMetaNotExists("xx".to_string());
        assert_eq!(
            get_journal_server_code(&e),
            "SegmentMetaNotExists".to_string()
        );
    }
}
