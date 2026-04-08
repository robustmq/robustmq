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
use crate::jstream::protocol::{DirectGetHeaders, DirectGetRequest};
use bytes::Bytes;

/// `$JS.API.DIRECT.GET.<stream>`
/// Returns headers + raw payload; caller builds the HMSG response.
pub async fn process_direct_get(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: DirectGetRequest,
) -> Result<(DirectGetHeaders, Bytes), NatsBrokerError> {
    todo!("DIRECT.GET")
}

/// `$JS.API.DIRECT.GET.<stream>.<subject>` — get latest message for subject
pub async fn process_direct_get_by_subject(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _subject: &str,
) -> Result<(DirectGetHeaders, Bytes), NatsBrokerError> {
    todo!("DIRECT.GET (by subject)")
}

/// `$JS.API.DIRECT.GET.LAST.<stream>.<subject>` — alias for last-by-subject
pub async fn process_direct_get_last(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _subject: &str,
) -> Result<(DirectGetHeaders, Bytes), NatsBrokerError> {
    todo!("DIRECT.GET.LAST")
}
