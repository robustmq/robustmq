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

use crate::raft::route::AppResponseData;
use crate::raft::route::DataRoute;
use crate::raft::store::snapshot::build_snapshot;
use crate::raft::store::snapshot::get_current_snapshot_;
use crate::raft::store::snapshot::recover_snapshot;
use crate::raft::store::snapshot::set_current_snapshot_;
use crate::raft::store::snapshot::StoredSnapshot;
use crate::raft::type_config::Entry;
use crate::raft::type_config::{SnapshotData, TypeConfig};
use bytes::Bytes;
use common_base::tools::now_nanos;
use openraft::storage::RaftStateMachine;
use openraft::{
    EntryPayload, LogId, OptionalSend, RaftSnapshotBuilder, Snapshot, SnapshotMeta, StorageError,
    StoredMembership,
};
use std::io::Cursor;
use std::sync::Arc;

#[derive(Clone)]
pub struct StateMachineStore {
    pub data: StateMachineData,
    pub machine: String,
}

#[derive(Clone)]
pub struct StateMachineData {
    pub last_applied_log_id: Option<LogId<TypeConfig>>,

    pub last_membership: StoredMembership<TypeConfig>,

    pub route: Arc<DataRoute>,
}

impl StateMachineStore {
    pub async fn new(
        machine: String,
        route: Arc<DataRoute>,
    ) -> Result<StateMachineStore, StorageError<TypeConfig>> {
        Ok(Self {
            machine,
            data: StateMachineData {
                last_applied_log_id: None,
                last_membership: Default::default(),
                route,
            },
        })
    }
}

impl RaftSnapshotBuilder<TypeConfig> for StateMachineStore {
    async fn build_snapshot(&mut self) -> Result<Snapshot<TypeConfig>, StorageError<TypeConfig>> {
        let last_applied_log = self.data.last_applied_log_id;
        let last_membership = self.data.last_membership.clone();

        let kv_json = build_snapshot(&self.machine);

        let snapshot_id = if let Some(last) = last_applied_log {
            format!("{}-{}-{}", last.leader_id, last.index, now_nanos())
        } else {
            format!("--{}", now_nanos())
        };

        let meta = SnapshotMeta {
            last_log_id: last_applied_log,
            last_membership,
            snapshot_id,
        };

        let snapshot = StoredSnapshot {
            meta: meta.clone(),
            data: kv_json.clone(),
        };

        set_current_snapshot_(snapshot)?;

        Ok(Snapshot {
            meta,
            snapshot: Cursor::new(kv_json.to_vec()),
        })
    }
}

impl RaftStateMachine<TypeConfig> for StateMachineStore {
    type SnapshotBuilder = Self;

    async fn applied_state(
        &mut self,
    ) -> Result<(Option<LogId<TypeConfig>>, StoredMembership<TypeConfig>), StorageError<TypeConfig>>
    {
        Ok((
            self.data.last_applied_log_id,
            self.data.last_membership.clone(),
        ))
    }

    async fn apply<I>(
        &mut self,
        entries: I,
    ) -> Result<Vec<AppResponseData>, StorageError<TypeConfig>>
    where
        I: IntoIterator<Item = Entry> + OptionalSend,
        I::IntoIter: OptionalSend,
    {
        let entries = entries.into_iter();
        let mut replies = Vec::with_capacity(entries.size_hint().0);

        for ent in entries {
            let mut resp_value = None;

            // Process the entry BEFORE updating last_applied_log_id
            match ent.payload {
                EntryPayload::Blank => {}
                EntryPayload::Normal(req) => match self.data.route.route(&req).await {
                    Ok(data) => {
                        resp_value = data;
                    }
                    Err(e) => {
                        use tracing::error;
                        error!(
                            "[{}] Failed to apply log {}: {}, req type: {:?}",
                            self.machine, ent.log_id.index, e, req.data_type
                        );
                        return Err(StorageError::write(&e));
                    }
                },
                EntryPayload::Membership(mem) => {
                    self.data.last_membership = StoredMembership::new(Some(ent.log_id), mem);
                }
            }

            // Only update last_applied_log_id AFTER successful processing
            self.data.last_applied_log_id = Some(ent.log_id);

            replies.push(AppResponseData { value: resp_value });
        }
        Ok(replies)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Cursor<Vec<u8>>, StorageError<TypeConfig>> {
        Ok(Cursor::new(Vec::new()))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<TypeConfig>,
        snapshot: SnapshotData,
    ) -> Result<(), StorageError<TypeConfig>> {
        let new_snapshot = StoredSnapshot {
            meta: meta.clone(),
            data: Bytes::copy_from_slice(&snapshot.into_inner()),
        };

        recover_snapshot(new_snapshot.clone())?;

        set_current_snapshot_(new_snapshot)?;

        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<TypeConfig>>, StorageError<TypeConfig>> {
        let x = get_current_snapshot_()?;
        Ok(x.map(|s| Snapshot {
            meta: s.meta.clone(),
            snapshot: Cursor::new(s.data.to_vec()),
        }))
    }
}
