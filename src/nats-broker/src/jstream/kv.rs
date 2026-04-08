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

pub async fn process_kv_put(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
    _payload: &Bytes,
) -> Result<String, NatsBrokerError> {
    todo!("KV PUT")
}

pub async fn process_kv_get(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
) -> Result<String, NatsBrokerError> {
    todo!("KV GET")
}

pub async fn process_kv_delete(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
) -> Result<String, NatsBrokerError> {
    todo!("KV DELETE")
}

pub async fn process_kv_purge(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
) -> Result<String, NatsBrokerError> {
    todo!("KV PURGE")
}

pub async fn process_kv_keys(
    _ctx: &NatsProcessContext,
    _bucket: &str,
) -> Result<String, NatsBrokerError> {
    todo!("KV KEYS")
}

pub async fn process_kv_watch(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _pattern: &str,
) -> Result<(), NatsBrokerError> {
    todo!("KV WATCH")
}
