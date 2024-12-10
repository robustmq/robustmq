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

/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io;
use std::net::AddrParseError;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("{0}")]
    FromTonicTransport(#[from] tonic::transport::Error),

    #[error("{0}")]
    FromErrorKind(#[from] Box<bincode::ErrorKind>),

    #[error("{0}")]
    FromDecodeError(#[from] prost::DecodeError),

    #[error("{0}")]
    FromSerdeJsonError(#[from] serde_json::Error),

    #[error("{0}")]
    FromRocksdbError(#[from] rocksdb::Error),

    #[error("{0}")]
    FromIoError(#[from] io::Error),

    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("{0}")]
    FromAddrParseError(#[from] AddrParseError),

    #[error("{0}")]
    FromMysqlError(#[from] mysql::Error),

    #[error("{0}")]
    FromParseIntError(#[from] ParseIntError),

    #[error("{0}")]
    CommonError(String),

    #[error("Grpc call of the node failed,Grpc status was {0}")]
    GrpcServerStatus(#[from] Status),

    #[error("{0} connection pool has no connection information available. {1}")]
    NoAvailableGrpcConnection(String, String),

    #[error("Parameter cannot be empty, parameter name: {0}")]
    ParameterCannotBeNull(String),

    #[error("Invalid parameter format, parameter name: {0}, parameter_value: {1}")]
    InvalidParameterFormat(String, String),

    #[error("Module {0} does not support this feature {1}")]
    NotSupportFeature(String, String),

    #[error("Unavailable cluster type")]
    UnavailableClusterType,

    #[error("No available nodes in the cluster")]
    ClusterNoAvailableNode,

    #[error("RocksDB Family {0} not available")]
    RocksDBFamilyNotAvailable(String),
}

impl From<CommonError> for Status {
    fn from(e: CommonError) -> Self {
        Status::cancelled(e.to_string())
    }
}
