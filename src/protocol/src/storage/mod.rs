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

#![cfg_attr(any(), rustfmt::skip)]
#![allow(clippy::all)]

use std::io;

pub mod storage_engine_engine {
    tonic::include_proto!("storage.engine");
}

pub mod codec;

/// Error during serialization and deserialization
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("data store disconnected")]
    IoError(#[from] io::Error),
    #[error("Payload size has been exceeded by {0} bytes")]
    PayloadSizeLimitExceeded(usize),
    #[error("Length of the request header is 0")]
    HeaderLengthIsZero,
    #[error("Error parsing request header, error message {0}")]
    DecodeHeaderError(String),
    #[error("Parsing request body error, request body identification: {0}, error message {1}")]
    DecodeBodyError(String, String),
    #[error("Type {0} is an unavailable request type")]
    NotAvailableRequestType(u8),
}
