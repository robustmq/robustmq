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

use crate::core::error::MetaServiceError;
use crate::raft::manager::RaftStateMachineName;
use crate::raft::type_config::{StorageResult, TypeConfig};
use bincode::{deserialize, serialize};
use common_base::tools::now_nanos;
use common_config::broker::broker_config;
use openraft::{LogId, Snapshot, SnapshotMeta, StorageError, StoredMembership};
use rocksdb::DB;
use rocksdb_engine::storage::family::storage_raft_snapshot_fold;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::error;

pub async fn save_last_snapshot_id(machine: &str, last_snapshot_id: &str) -> std::io::Result<()> {
    save_last_snapshot_id_to_path(machine, last_snapshot_id, None).await
}

pub async fn get_last_snapshot_id(machine: &str) -> std::io::Result<String> {
    get_last_snapshot_id_from_path(machine, None).await
}

async fn save_last_snapshot_id_to_path(
    machine: &str,
    last_snapshot_id: &str,
    custom_path: Option<&str>,
) -> std::io::Result<()> {
    let file_path = if let Some(path) = custom_path {
        format!("{}/{}.last_snapshot_id", path, machine)
    } else {
        machine_last_snapshot_id(machine)
    };

    let mut file = File::create(&file_path).await?;
    file.write_all(last_snapshot_id.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

async fn get_last_snapshot_id_from_path(
    machine: &str,
    custom_path: Option<&str>,
) -> std::io::Result<String> {
    let file_path = if let Some(path) = custom_path {
        format!("{}/{}.last_snapshot_id", path, machine)
    } else {
        machine_last_snapshot_id(machine)
    };

    let mut file = File::open(&file_path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    Ok(content)
}

pub async fn save_snapshot_meta(meta: SnapshotMeta<TypeConfig>) -> std::io::Result<()> {
    save_snapshot_meta_to_path(meta, None).await
}

pub async fn get_snapshot_meta(snapshot_id: &str) -> std::io::Result<SnapshotMeta<TypeConfig>> {
    get_snapshot_meta_from_path(snapshot_id, None).await
}

async fn save_snapshot_meta_to_path(
    meta: SnapshotMeta<TypeConfig>,
    custom_path: Option<&str>,
) -> std::io::Result<()> {
    let file_path = if let Some(path) = custom_path {
        format!("{}/{}.meta", path, meta.snapshot_id)
    } else {
        snapshot_meta(&meta.snapshot_id)
    };

    let data = serialize(&meta).map_err(|e| {
        std::io::Error::new(ErrorKind::InvalidData, format!("Serialize error: {}", e))
    })?;

    let mut file = File::create(&file_path).await?;
    file.write_all(&data).await?;
    file.flush().await?;
    Ok(())
}

async fn get_snapshot_meta_from_path(
    snapshot_id: &str,
    custom_path: Option<&str>,
) -> std::io::Result<SnapshotMeta<TypeConfig>> {
    let file_path = if let Some(path) = custom_path {
        format!("{}/{}.meta", path, snapshot_id)
    } else {
        snapshot_meta(snapshot_id)
    };

    let mut file = File::open(&file_path).await?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).await?;

    deserialize(&data).map_err(|e| {
        std::io::Error::new(ErrorKind::InvalidData, format!("Deserialize error: {}", e))
    })
}

pub async fn build_snapshot(
    machine: &RaftStateMachineName,
    db: &Arc<DB>,
    last_applied_log_id: &Option<LogId<TypeConfig>>,
    last_membership: &StoredMembership<TypeConfig>,
) -> StorageResult<Snapshot<TypeConfig>> {
    let snapshot_id = format!("{}-{}", machine, now_nanos());
    let meta = SnapshotMeta {
        last_log_id: *last_applied_log_id,
        last_membership: last_membership.clone(),
        snapshot_id: snapshot_id.clone(),
    };
    let row_db = db.clone();
    let row_meta = meta.clone();
    let row_snapshot_id = snapshot_id.clone();
    let row_machine = machine.as_str().to_string();
    let row_machine_enum = machine.clone();
    tokio::spawn(async move {
        let snapshot_db = row_db.snapshot();

        let res = match row_machine_enum {
            RaftStateMachineName::METADATA => build_snapshot_by_metadata().await,
            RaftStateMachineName::OFFSET => build_snapshot_by_mqtt().await,
            RaftStateMachineName::MQTT => build_snapshot_by_offset().await,
        };

        if let Err(e) = res {
            error!(
                "[{}] Failed to build snapshot data for snapshot_id={}: {}",
                row_machine, row_snapshot_id, e
            );
        }

        if let Err(e) = save_snapshot_meta(row_meta).await {
            error!(
                "[{}] Failed to save snapshot metadata for snapshot_id={}: {}",
                row_machine, row_snapshot_id, e
            );
        }

        if let Err(e) = save_last_snapshot_id(&row_machine, &row_snapshot_id).await {
            error!(
                "[{}] Failed to save last snapshot id for snapshot_id={}: {}",
                row_machine, row_snapshot_id, e
            );
        }

        drop(snapshot_db);
    });

    let res = File::open(&snapshot_name(&snapshot_id))
        .await
        .map_err(|e| StorageError::read(&e))?;

    Ok(Snapshot {
        meta,
        snapshot: res,
    })
}

async fn build_snapshot_by_metadata() -> Result<(), MetaServiceError> {
    Ok(())
}

async fn build_snapshot_by_mqtt() -> Result<(), MetaServiceError> {
    Ok(())
}

async fn build_snapshot_by_offset() -> Result<(), MetaServiceError> {
    Ok(())
}

pub fn recover_snapshot(
    machine: &RaftStateMachineName,
    _snapshot: Snapshot<TypeConfig>,
) -> StorageResult<()> {
    match machine {
        RaftStateMachineName::METADATA => {}
        RaftStateMachineName::OFFSET => {}
        RaftStateMachineName::MQTT => {}
    };

    Ok(())
}

pub async fn get_current_snapshot_(machine: &str) -> StorageResult<Option<Snapshot<TypeConfig>>> {
    let snapshot_id = get_last_snapshot_id(machine)
        .await
        .map_err(|e| StorageError::read(&e))?;

    let snapshot_meta = get_snapshot_meta(&snapshot_id)
        .await
        .map_err(|e| StorageError::read(&e))?;

    let res = File::open(&snapshot_name(&snapshot_id))
        .await
        .map_err(|e| StorageError::read(&e))?;

    Ok(Some(Snapshot {
        meta: snapshot_meta,
        snapshot: res,
    }))
}

pub fn snapshot_name(snapshot_id: &str) -> String {
    let conf = broker_config();
    format!(
        "{}/{}.bin",
        storage_raft_snapshot_fold(&conf.rocksdb.data_path),
        snapshot_id
    )
}

pub fn snapshot_meta(snapshot_id: &str) -> String {
    let conf = broker_config();
    format!(
        "{}/{}.meta",
        storage_raft_snapshot_fold(&conf.rocksdb.data_path),
        snapshot_id
    )
}

pub fn machine_last_snapshot_id(machine: &str) -> String {
    let conf = broker_config();
    format!(
        "{}/{}.last_snapshot_id",
        storage_raft_snapshot_fold(&conf.rocksdb.data_path),
        machine
    )
}

#[cfg(test)]
mod tests {
    use super::{
        get_last_snapshot_id_from_path, get_snapshot_meta_from_path, save_last_snapshot_id_to_path,
        save_snapshot_meta_to_path,
    };
    use crate::raft::type_config::{Node, TypeConfig};
    use common_base::utils::file_utils::test_temp_dir;
    use openraft::vote::leader_id_adv::LeaderId;
    use openraft::{LogId, Membership, SnapshotMeta, StoredMembership};
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs::create_dir_all;

    #[tokio::test]
    async fn test_snapshot_meta() {
        let test_dir = format!("{}/snapshot_meta_test", test_temp_dir());
        create_dir_all(&test_dir).unwrap();

        let snapshot_id = "test-snapshot-123456789".to_string();
        let log_id = LogId {
            leader_id: LeaderId {
                term: 1,
                node_id: 1,
            },
            index: 100,
        };

        let mut nodes = BTreeSet::new();
        nodes.insert(1);
        nodes.insert(2);

        let mut node_map = BTreeMap::new();
        node_map.insert(
            1,
            Node {
                node_id: 1,
                rpc_addr: "127.0.0.1:9001".to_string(),
            },
        );
        node_map.insert(
            2,
            Node {
                node_id: 2,
                rpc_addr: "127.0.0.1:9002".to_string(),
            },
        );

        let membership = Membership::new(vec![nodes], node_map).unwrap();
        let stored_membership = StoredMembership::new(Some(log_id), membership);

        let meta = SnapshotMeta::<TypeConfig> {
            last_log_id: Some(log_id),
            last_membership: stored_membership.clone(),
            snapshot_id: snapshot_id.clone(),
        };

        save_snapshot_meta_to_path(meta.clone(), Some(&test_dir))
            .await
            .unwrap();
        let loaded_meta = get_snapshot_meta_from_path(&snapshot_id, Some(&test_dir))
            .await
            .unwrap();

        assert_eq!(loaded_meta.snapshot_id, meta.snapshot_id);
        assert_eq!(loaded_meta.last_log_id, meta.last_log_id);
        assert_eq!(loaded_meta.last_membership, meta.last_membership);

        assert!(get_snapshot_meta_from_path("non-existent", Some(&test_dir))
            .await
            .is_err());

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[tokio::test]
    async fn test_last_snapshot_id() {
        let test_dir = format!("{}/last_snapshot_id_test", test_temp_dir());
        create_dir_all(&test_dir).unwrap();

        let machine = "metadata";
        let snapshot_id_1 = "metadata-1111111111111111111";
        let snapshot_id_2 = "metadata-2222222222222222222";

        save_last_snapshot_id_to_path(machine, snapshot_id_1, Some(&test_dir))
            .await
            .unwrap();
        assert_eq!(
            get_last_snapshot_id_from_path(machine, Some(&test_dir))
                .await
                .unwrap(),
            snapshot_id_1
        );

        save_last_snapshot_id_to_path(machine, snapshot_id_2, Some(&test_dir))
            .await
            .unwrap();
        assert_eq!(
            get_last_snapshot_id_from_path(machine, Some(&test_dir))
                .await
                .unwrap(),
            snapshot_id_2
        );

        assert!(
            get_last_snapshot_id_from_path("non-existent", Some(&test_dir))
                .await
                .is_err()
        );

        std::fs::remove_dir_all(&test_dir).ok();
    }
}
