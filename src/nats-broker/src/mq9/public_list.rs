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
use mq9_core::protocol::{ListPublicMailboxReply, Mq9Reply};

pub async fn process_public_list(
    _ctx: &NatsProcessContext,
    _payload: &Bytes,
) -> Result<Mq9Reply, NatsBrokerError> {
    Ok(Mq9Reply::PublicList(ListPublicMailboxReply {
        items: vec![],
    }))
}
