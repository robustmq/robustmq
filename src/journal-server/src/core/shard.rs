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

use common_base::config::journal_server::journal_server_conf;
use grpc_clients::poll::ClientPool;
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
use metadata_struct::journal::segment::{JournalSegment, JournalSegmentStatus};
use metadata_struct::journal::shard::JournalShard;
use protocol::placement_center::placement_center_journal::CreateShardRequest;

use super::error::JournalServerError;

pub async fn create_shard(
    client_poll: Arc<ClientPool>,
    namespace: &str,
    shard_name: &str,
) -> Result<JournalShard, JournalServerError> {
    let conf = journal_server_conf();
    let request = CreateShardRequest {
        cluster_name: conf.cluster_name.to_string(),
        namespace: namespace.to_string(),
        shard_name: shard_name.to_string(),
        replica: 3,
    };
    let reply = grpc_clients::placement::journal::call::create_shard(
        client_poll,
        conf.placement_center.clone(),
        request,
    )
    .await?;
    Ok(JournalShard::default())
}

pub fn delete_shard() -> Result<(), JournalServerError> {
    Ok(())
}

pub async fn create_active_segement(
    namespace: &str,
    shard_name: &str,
) -> Result<JournalSegment, JournalServerError> {
    let segment = JournalSegment {
        shard_name: shard_name.to_string(),
        segment_seq: 0,
        start_offset: None,
        end_offset: None,
        replica: Vec::new(),
        status: JournalSegmentStatus::CREATE,
    };
    Ok(segment)
}
