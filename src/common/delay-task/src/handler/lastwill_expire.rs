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
use node_call::{NodeCallData, NodeCallManager};
use std::sync::Arc;
use tracing::debug;

pub async fn handle_lastwill_expire(
    node_call_manager: &Arc<NodeCallManager>,
    client_id: &str,
) -> Result<(), CommonError> {
    node_call_manager
        .send(NodeCallData::SendLastWillMessage(client_id.to_string()))
        .await?;
    debug!(
        "Lastwill expire handling completed: client_id={}",
        client_id
    );
    Ok(())
}
