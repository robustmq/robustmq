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

pub async fn process_consumer_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.CREATE")
}

pub async fn process_consumer_create_named(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.CREATE (named)")
}

pub async fn process_consumer_durable_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.DURABLE.CREATE")
}

pub async fn process_consumer_delete(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.DELETE")
}

pub async fn process_consumer_info(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.INFO")
}

pub async fn process_consumer_list(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.LIST")
}

pub async fn process_consumer_names(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.NAMES")
}

pub async fn process_consumer_msg_next(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.MSG.NEXT")
}

pub async fn process_consumer_leader_stepdown(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.LEADER.STEPDOWN")
}

pub async fn process_consumer_pause(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("CONSUMER.PAUSE")
}
