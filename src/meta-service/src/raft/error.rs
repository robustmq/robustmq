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

use super::type_config::{Node, NodeId};

pub fn to_error<E: std::error::Error + 'static + Clone>(
    e: CommonError,
) -> RPCError<NodeId, Node, E> {
    RPCError::Unreachable(Unreachable::new(&e))
}

pub fn to_rpc_error<E: std::error::Error + 'static + Clone>(
    err: impl Display,
) -> RPCError<NodeId, Node, E> {
    let common_err = CommonError::CommonError(err.to_string());
    RPCError::Unreachable(Unreachable::new(&common_err))
}

pub fn to_bincode_error<E: std::error::Error + 'static + Clone>(
    err: bincode::Error,
    context: &str,
) -> RPCError<NodeId, Node, E> {
    let msg = format!("{}: {}", context, err);
    let common_err = CommonError::CommonError(msg);
    RPCError::Unreachable(Unreachable::new(&common_err))
}

pub fn to_grpc_error<E: std::error::Error + 'static + Clone>(
    err: impl Display,
    context: &str,
) -> RPCError<NodeId, Node, E> {
    let msg = format!("{}: {}", context, err);
    let common_err = CommonError::CommonError(msg);
    RPCError::Unreachable(Unreachable::new(&common_err))
}
