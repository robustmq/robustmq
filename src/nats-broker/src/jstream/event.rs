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

pub async fn process_event_stream_created(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.CREATED")
}

pub async fn process_event_stream_deleted(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.DELETED")
}

pub async fn process_event_stream_updated(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.UPDATED")
}

pub async fn process_event_consumer_created(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.CONSUMER.CREATED")
}

pub async fn process_event_consumer_deleted(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.CONSUMER.DELETED")
}

pub async fn process_event_api_audit(
    _ctx: &NatsProcessContext,
    _subject: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.API")
}

pub async fn process_event_snapshot_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.SNAPSHOT.CREATE")
}

pub async fn process_event_snapshot_complete(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.SNAPSHOT.COMPLETE")
}

pub async fn process_event_restore_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.RESTORE.CREATE")
}

pub async fn process_event_restore_complete(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.RESTORE.COMPLETE")
}

pub async fn process_event_consumer_leader_elected(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.CONSUMER.LEADER_ELECTED")
}

pub async fn process_event_stream_leader_elected(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.LEADER_ELECTED")
}
