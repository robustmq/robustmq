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

pub async fn process_stream_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.CREATE")
}

pub async fn process_stream_update(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.UPDATE")
}

pub async fn process_stream_delete(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.DELETE")
}

pub async fn process_stream_info(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.INFO")
}

pub async fn process_stream_list(
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.LIST")
}

pub async fn process_stream_names(
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.NAMES")
}

pub async fn process_stream_purge(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.PURGE")
}

pub async fn process_stream_msg_get(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.MSG.GET")
}

pub async fn process_stream_msg_delete(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.MSG.DELETE")
}

pub async fn process_stream_snapshot(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.SNAPSHOT")
}

pub async fn process_stream_restore(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.RESTORE")
}

pub async fn process_stream_leader_stepdown(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.LEADER.STEPDOWN")
}

pub async fn process_stream_peer_remove(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("STREAM.PEER.REMOVE")
}
