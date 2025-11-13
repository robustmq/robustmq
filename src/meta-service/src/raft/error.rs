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
use openraft::error::{RPCError, Unreachable};
use std::fmt::Display;

use super::type_config::TypeConfig;

// Convert CommonError to RPCError with proper error classification
pub fn to_error<E: std::error::Error + 'static + Clone>(e: CommonError) -> RPCError<TypeConfig, E> {
    RPCError::Unreachable(Unreachable::new(&e))
}

// Convert any error that implements Display to RPCError
pub fn to_rpc_error<E: std::error::Error + 'static + Clone>(
    err: impl Display,
) -> RPCError<TypeConfig, E> {
    let common_err = CommonError::CommonError(err.to_string());
    RPCError::Unreachable(Unreachable::new(&common_err))
}

// Convert bincode errors with context
pub fn to_bincode_error<E: std::error::Error + 'static + Clone>(
    err: bincode::Error,
    context: &str,
) -> RPCError<TypeConfig, E> {
    let msg = format!("{}: {}", context, err);
    let common_err = CommonError::CommonError(msg);
    RPCError::Unreachable(Unreachable::new(&common_err))
}

// Convert gRPC errors with context
pub fn to_grpc_error<E: std::error::Error + 'static + Clone>(
    err: impl Display,
    context: &str,
) -> RPCError<TypeConfig, E> {
    let msg = format!("{}: {}", context, err);
    let common_err = CommonError::CommonError(msg);
    RPCError::Unreachable(Unreachable::new(&common_err))
}
