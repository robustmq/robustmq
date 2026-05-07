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
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use mq9_core::protocol::Mq9Reply;

pub async fn process_delete(
    ctx: &NatsProcessContext,
    mail_address: &str,
    msg_id: &str,
) -> Result<Mq9Reply, NatsBrokerError> {
    let offset: u64 = msg_id
        .parse()
        .map_err(|_| NatsBrokerError::CommonError(format!("invalid msg_id: {}", msg_id)))?;

    let tenant = get_tenant();
    ctx.storage_driver_manager
        .delete_by_offsets(&tenant, mail_address, &[offset])
        .await
        .map_err(NatsBrokerError::from)?;

    Ok(Mq9Reply::ok_delete())
}
