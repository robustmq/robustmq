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

use crate::core::error::NatsBrokerError;
use crate::handler::command::NatsProcessContext;
use bytes::Bytes;

/// Upload an object in chunks to the given bucket.
pub async fn process_obj_put(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("OBJ PUT")
}

pub async fn process_obj_get(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _object: &str,
) -> Result<String, NatsBrokerError> {
    todo!("OBJ GET")
}

pub async fn process_obj_delete(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _object: &str,
) -> Result<String, NatsBrokerError> {
    todo!("OBJ DELETE")
}

pub async fn process_obj_info(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _object: &str,
) -> Result<String, NatsBrokerError> {
    todo!("OBJ INFO")
}

pub async fn process_obj_list(
    _ctx: &NatsProcessContext,
    _bucket: &str,
) -> Result<String, NatsBrokerError> {
    todo!("OBJ LIST")
}

pub async fn process_obj_watch(
    _ctx: &NatsProcessContext,
    _bucket: &str,
) -> Result<(), NatsBrokerError> {
    todo!("OBJ WATCH")
}
