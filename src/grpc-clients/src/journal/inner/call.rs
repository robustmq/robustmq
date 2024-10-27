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

use std::sync::Arc;

use common_base::error::common::CommonError;
use prost::Message as _;
use protocol::journal_server::journal_inner::{UpdateJournalCacheReply, UpdateJournalCacheRequest};

use crate::journal::{retry_call, JournalEngineInterface, JournalEngineService};
use crate::pool::ClientPool;

pub async fn journal_inner_update_cache(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateJournalCacheRequest,
) -> Result<UpdateJournalCacheReply, CommonError> {
    let request_data = UpdateJournalCacheRequest::encode_to_vec(&request);
    match retry_call(
        JournalEngineService::Inner,
        JournalEngineInterface::UpdateCache,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match UpdateJournalCacheReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
