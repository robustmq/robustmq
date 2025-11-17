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

use crate::raft::manager::RaftStateMachineName;
use crate::raft::route::AppResponseData;
use crate::raft::route::DataRoute;
use crate::raft::store::keys::{key_last_applied, key_last_membership};
use crate::raft::store::snapshot::build_snapshot;
use crate::raft::store::snapshot::get_current_snapshot_;
use crate::raft::store::snapshot::recover_snapshot;
use crate::raft::type_config::Entry;
use crate::raft::type_config::{SnapshotData, StorageResult, TypeConfig};
use bincode::{deserialize, serialize};
use common_base::error::common::CommonError;
use openraft::storage::RaftStateMachine;
use openraft::{
    AnyError, EntryPayload, LogId, OptionalSend, RaftSnapshotBuilder, Snapshot, SnapshotMeta,
    StorageError, StoredMembership,
};
use rocksdb::{BoundColumnFamily, DB};
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_META_RAFT;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct StateMachineStore {
    pub data: StateMachineData,
    pub machine: String,
    pub db: Arc<DB>,
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
        db: Arc<DB>,
        route: Arc<DataRoute>,
    ) -> Result<StateMachineStore, StorageError<TypeConfig>> {
        let mut sm = Self {
            machine: machine.clone(),
            db: db.clone(),
            data: StateMachineData {
                last_applied_log_id: None,
                last_membership: Default::default(),
                route,
            },
        };

        // Recover state from persistent storage
        sm.data.last_applied_log_id = sm.get_last_applied_()?;
        sm.data.last_membership = sm.get_last_membership_()?.unwrap_or_default();

        Ok(sm)
    }

    #[inline]
    fn store(&self) -> Arc<BoundColumnFamily<'_>> {
        self.db.cf_handle(DB_COLUMN_FAMILY_META_RAFT).unwrap()
    }

    fn get_last_applied_(&self) -> StorageResult<Option<LogId<TypeConfig>>> {
        match self
            .db
            .get_cf(&self.store(), key_last_applied(&self.machine))
        {
            Ok(Some(v)) => {
                let log_id = deserialize(&v).map_err(|e| {
                    StorageError::read(AnyError::error(format!(
                        "Failed to deserialize last_applied: {}",
                        e
                    )))
                })?;
                Ok(Some(log_id))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::read(&e)),
        }
    }

    fn set_last_applied_(&self, log_id: Option<LogId<TypeConfig>>) -> StorageResult<()> {
        match log_id {
            Some(id) => {
                let data = serialize(&id).map_err(|e| StorageError::write(&e))?;
                self.db
                    .put_cf(&self.store(), key_last_applied(&self.machine), data)
                    .map_err(|e| StorageError::write(&e))
            }
            None => self
                .db
                .delete_cf(&self.store(), key_last_applied(&self.machine))
                .map_err(|e| StorageError::write(&e)),
        }
    }

    fn get_last_membership_(&self) -> StorageResult<Option<StoredMembership<TypeConfig>>> {
        match self
            .db
            .get_cf(&self.store(), key_last_membership(&self.machine))
        {
            Ok(Some(v)) => {
                let membership = deserialize(&v).map_err(|e| {
                    StorageError::read(AnyError::error(format!(
                        "Failed to deserialize last_membership: {}",
                        e
                    )))
                })?;
                Ok(Some(membership))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::read(&e)),
        }
    }

    fn set_last_membership_(&self, membership: &StoredMembership<TypeConfig>) -> StorageResult<()> {
        let data = serialize(membership).map_err(|e| StorageError::write(&e))?;
        self.db
            .put_cf(&self.store(), key_last_membership(&self.machine), data)
            .map_err(|e| StorageError::write(&e))
    }
}

impl RaftSnapshotBuilder<TypeConfig> for StateMachineStore {
    async fn build_snapshot(&mut self) -> Result<Snapshot<TypeConfig>, StorageError<TypeConfig>> {
        let machine_name = self.machine.parse::<RaftStateMachineName>().map_err(|e| {
            StorageError::read(&CommonError::CommonError(format!(
                "Invalid machine name {}: {}",
                self.machine, e
            )))
        })?;

        build_snapshot(
            &machine_name,
            &self.db,
            &self.data.last_applied_log_id,
            &self.data.last_membership,
        )
        .await
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
                    // Persist membership change immediately (critical for cluster safety)
                    self.set_last_membership_(&self.data.last_membership)?;
                }
            }

            // Only update last_applied_log_id AFTER successful processing
            self.data.last_applied_log_id = Some(ent.log_id);

            replies.push(AppResponseData { value: resp_value });
        }

        // Persist last_applied_log_id after all entries are successfully applied
        if let Some(last_log_id) = self.data.last_applied_log_id {
            self.set_last_applied_(Some(last_log_id))?;
        }

        Ok(replies)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }

    async fn begin_receiving_snapshot(&mut self) -> Result<SnapshotData, StorageError<TypeConfig>> {
        let data = get_current_snapshot_(&self.machine)
            .await
            .map_err(|e| StorageError::read(&e))?;
        match data {
            Some(da) => Ok(da.snapshot),
            None => Err(StorageError::read(&CommonError::CommonError(
                "".to_string(),
            ))),
        }
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<TypeConfig>,
        snapshot: SnapshotData,
    ) -> Result<(), StorageError<TypeConfig>> {
        let machine_name = self.machine.parse::<RaftStateMachineName>().map_err(|e| {
            StorageError::read(&CommonError::CommonError(format!(
                "Invalid machine name {}: {}",
                self.machine, e
            )))
        })?;

        recover_snapshot(
            &machine_name,
            &self.db,
            Snapshot {
                meta: meta.clone(),
                snapshot,
            },
        )
        .await?;

        self.data.last_applied_log_id = meta.last_log_id;
        self.data.last_membership = meta.last_membership.clone();

        if let Some(log_id) = meta.last_log_id {
            self.set_last_applied_(Some(log_id))?;
        }
        self.set_last_membership_(&meta.last_membership)?;

        info!(
            "[{}] Snapshot installed, updated last_applied_log_id to {:?}",
            self.machine, meta.last_log_id
        );

        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<TypeConfig>>, StorageError<TypeConfig>> {
        let data = get_current_snapshot_(&self.machine)
            .await
            .map_err(|e| StorageError::read(&e))?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raft::route::DataRoute;
    use crate::raft::type_config::Node;
    use common_base::utils::file_utils::test_temp_dir;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use openraft::vote::leader_id_adv::LeaderId;
    use openraft::{LogId, Membership};
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use rocksdb_engine::storage::family::column_family_list;
    use std::collections::{BTreeMap, BTreeSet};

    fn setup_test_environment() -> (Arc<RocksDBEngine>, Arc<DataRoute>) {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());

        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));

        let route = Arc::new(DataRoute::new(
            rocksdb_engine.clone(),
            Arc::new(crate::core::cache::CacheManager::new(
                rocksdb_engine.clone(),
            )),
        ));

        (rocksdb_engine, route)
    }

    async fn create_test_state_machine() -> StateMachineStore {
        let (rocksdb_engine, route) = setup_test_environment();

        StateMachineStore::new("test_machine".to_string(), rocksdb_engine.db.clone(), route)
            .await
            .unwrap()
    }

    fn create_log_id(term: u64, node_id: u64, index: u64) -> LogId<TypeConfig> {
        LogId {
            leader_id: LeaderId { term, node_id },
            index,
        }
    }

    fn create_stored_membership(log_id: LogId<TypeConfig>) -> StoredMembership<TypeConfig> {
        let mut nodes = BTreeSet::new();
        nodes.insert(1);

        let mut node_map = BTreeMap::new();
        node_map.insert(
            1,
            Node {
                node_id: 1,
                rpc_addr: "127.0.0.1:1228".to_string(),
            },
        );

        let membership = Membership::new(vec![nodes], node_map).unwrap();
        StoredMembership::new(Some(log_id), membership)
    }

    #[tokio::test]
    async fn test_set_and_get_last_applied() {
        let sm = create_test_state_machine().await;

        assert!(sm.get_last_applied_().unwrap().is_none());

        let log_id = create_log_id(1, 1, 100);
        sm.set_last_applied_(Some(log_id)).unwrap();

        let retrieved = sm.get_last_applied_().unwrap().unwrap();
        assert_eq!(retrieved.leader_id.term, log_id.leader_id.term);
        assert_eq!(retrieved.leader_id.node_id, log_id.leader_id.node_id);
        assert_eq!(retrieved.index, log_id.index);

        let new_log_id = create_log_id(2, 2, 200);
        sm.set_last_applied_(Some(new_log_id)).unwrap();

        let updated = sm.get_last_applied_().unwrap().unwrap();
        assert_eq!(updated.leader_id.term, 2);
        assert_eq!(updated.leader_id.node_id, 2);
        assert_eq!(updated.index, 200);

        sm.set_last_applied_(None).unwrap();
        assert!(sm.get_last_applied_().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_last_membership() {
        let sm = create_test_state_machine().await;

        assert!(sm.get_last_membership_().unwrap().is_none());

        let log_id = create_log_id(1, 1, 100);
        let membership = create_stored_membership(log_id);
        sm.set_last_membership_(&membership).unwrap();

        let retrieved = sm.get_last_membership_().unwrap().unwrap();
        assert_eq!(retrieved.log_id(), membership.log_id());
        assert_eq!(
            retrieved.membership().get_joint_config().len(),
            membership.membership().get_joint_config().len()
        );

        let new_log_id = create_log_id(2, 2, 200);
        let new_membership = create_stored_membership(new_log_id);
        sm.set_last_membership_(&new_membership).unwrap();

        let updated = sm.get_last_membership_().unwrap().unwrap();
        assert_eq!(updated.log_id().unwrap().leader_id.term, 2);
        assert_eq!(updated.log_id().unwrap().index, 200);
    }

    #[tokio::test]
    async fn test_state_machine_recovery() {
        let (rocksdb_engine, route) = setup_test_environment();

        let log_id = create_log_id(1, 1, 100);
        let membership = create_stored_membership(log_id);

        // Simulate node shutdown: persist state in first instance
        {
            let sm = StateMachineStore::new(
                "test_machine".to_string(),
                rocksdb_engine.db.clone(),
                route.clone(),
            )
            .await
            .unwrap();

            sm.set_last_applied_(Some(log_id)).unwrap();
            sm.set_last_membership_(&membership).unwrap();
        }

        // Simulate node restart: create new instance with same DB
        let sm_recovered = StateMachineStore::new(
            "test_machine".to_string(),
            rocksdb_engine.db.clone(),
            route.clone(),
        )
        .await
        .unwrap();

        // Verify recovery
        let recovered_applied = sm_recovered.data.last_applied_log_id.unwrap();
        assert_eq!(recovered_applied.index, 100);
        assert_eq!(recovered_applied.leader_id.term, 1);

        let recovered_membership_log = sm_recovered.data.last_membership.log_id().unwrap();
        assert_eq!(recovered_membership_log.index, 100);
    }
}
