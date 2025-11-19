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

use super::{save_last_snapshot_id, save_snapshot_meta, snapshot_name};
use crate::core::error::MetaServiceError;
use crate::raft::manager::RaftStateMachineName;
use crate::raft::type_config::{StorageResult, TypeConfig};
use common_base::tools::now_nanos;
use common_config::broker::broker_config;
use openraft::{LogId, Snapshot, SnapshotMeta, StorageError, StoredMembership};
use rocksdb::{IteratorMode, DB};
use rocksdb_engine::storage::family::{
    storage_raft_snapshot_fold, DB_COLUMN_FAMILY_META_DATA, DB_COLUMN_FAMILY_META_METADATA,
};
use std::io::{ErrorKind, Write};
use std::sync::Arc;
use tokio::fs::File;
use tracing::{error, info};

pub async fn build_snapshot(
    machine: &RaftStateMachineName,
    db: &Arc<DB>,
    last_applied_log_id: &Option<LogId<TypeConfig>>,
    last_membership: &StoredMembership<TypeConfig>,
) -> StorageResult<Snapshot<TypeConfig>> {
    let snapshot_id = format!("{}-{}", machine, now_nanos());
    info!("[{}] Starting to build snapshot, snapshot_id={}", machine, snapshot_id);
    
    let meta = SnapshotMeta {
        last_log_id: *last_applied_log_id,
        last_membership: last_membership.clone(),
        snapshot_id: snapshot_id.clone(),
    };
    let snapshot_db = db.snapshot();
    let snapshot_name_path = snapshot_name(&snapshot_id);
    
    let res = match machine {
        RaftStateMachineName::METADATA => {
            build_snapshot_by_metadata(db, &snapshot_db, &snapshot_name_path).await
        }
        RaftStateMachineName::OFFSET => {
            build_snapshot_by_offset(db, &snapshot_db, &snapshot_name_path).await
        }
        RaftStateMachineName::MQTT => {
            build_snapshot_by_mqtt(db, &snapshot_db, &snapshot_name_path).await
        }
    };

    if let Err(e) = res {
        error!(
            "[{}] Failed to build snapshot data for snapshot_id={}: {}",
            machine, snapshot_id, e
        );
        return Err(StorageError::read(&std::io::Error::other(
            format!("Failed to build snapshot data: {}", e),
        )));
    }

    if let Err(e) = save_snapshot_meta(meta.clone()).await {
        error!(
            "[{}] Failed to save snapshot metadata for snapshot_id={}: {}",
            machine, snapshot_id, e
        );
        return Err(StorageError::read(&e));
    }

    if let Err(e) = save_last_snapshot_id(machine.as_str(), &snapshot_id).await {
        error!(
            "[{}] Failed to save last snapshot id for snapshot_id={}: {}",
            machine, snapshot_id, e
        );
        return Err(StorageError::read(&e));
    }

    let res = File::open(&snapshot_name(&snapshot_id))
        .await
        .map_err(|e| StorageError::read(&e))?;

    let row_machine = machine.as_str().to_string();
    tokio::spawn(async move {
        if let Err(e) = cleanup_old_snapshots(&row_machine) {
            error!("[{}] Failed to cleanup old snapshots: {}", row_machine, e);
        }
    });

    info!("[{}] Snapshot build completed successfully for snapshot_id={}", machine, snapshot_id);

    Ok(Snapshot {
        meta,
        snapshot: res,
    })
}

async fn build_snapshot_by_metadata(
    db: &Arc<DB>,
    snapshot_db: &rocksdb::Snapshot<'_>,
    snapshot_name: &str,
) -> Result<(), MetaServiceError> {
    let dumping_path = format!("{}.dumping", snapshot_name);
    
    if let Some(parent) = std::path::Path::new(&dumping_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cf_handle = db
        .cf_handle(DB_COLUMN_FAMILY_META_METADATA)
        .ok_or_else(|| {
            MetaServiceError::CommonError(format!(
                "Column family {} not found",
                DB_COLUMN_FAMILY_META_METADATA
            ))
        })?;

    let file = std::fs::File::create(&dumping_path)?;
    let mut encoder = zstd::Encoder::new(file, 3)?;

    let iterator = snapshot_db.iterator_cf(&cf_handle, IteratorMode::Start);
    for item in iterator {
        let (key, value) = item?;
        encoder.write_all(&(key.len() as u32).to_le_bytes())?;
        encoder.write_all(&key)?;
        encoder.write_all(&(value.len() as u32).to_le_bytes())?;
        encoder.write_all(&value)?;
    }

    encoder.finish()?;
    std::fs::rename(&dumping_path, snapshot_name)?;

    Ok(())
}

async fn build_snapshot_by_mqtt(
    db: &Arc<DB>,
    snapshot_db: &rocksdb::Snapshot<'_>,
    snapshot_name: &str,
) -> Result<(), MetaServiceError> {
    let dumping_path = format!("{}.dumping", snapshot_name);
    let prefix = b"/mqtt/";
    
    if let Some(parent) = std::path::Path::new(&dumping_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cf_handle = db.cf_handle(DB_COLUMN_FAMILY_META_DATA).ok_or_else(|| {
        MetaServiceError::CommonError(format!(
            "Column family {} not found",
            DB_COLUMN_FAMILY_META_DATA
        ))
    })?;

    let file = std::fs::File::create(&dumping_path)?;
    let mut encoder = zstd::Encoder::new(file, 3)?;

    let iterator = snapshot_db.iterator_cf(
        &cf_handle,
        IteratorMode::From(prefix, rocksdb::Direction::Forward),
    );
    for item in iterator {
        let (key, value) = item?;
        if !key.starts_with(prefix) {
            break;
        }
        encoder.write_all(&(key.len() as u32).to_le_bytes())?;
        encoder.write_all(&key)?;
        encoder.write_all(&(value.len() as u32).to_le_bytes())?;
        encoder.write_all(&value)?;
    }

    encoder.finish()?;
    std::fs::rename(&dumping_path, snapshot_name)?;

    Ok(())
}

async fn build_snapshot_by_offset(
    db: &Arc<DB>,
    snapshot_db: &rocksdb::Snapshot<'_>,
    snapshot_name: &str,
) -> Result<(), MetaServiceError> {
    let dumping_path = format!("{}.dumping", snapshot_name);
    let prefix = b"/offset/";
    
    if let Some(parent) = std::path::Path::new(&dumping_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cf_handle = db.cf_handle(DB_COLUMN_FAMILY_META_DATA).ok_or_else(|| {
        MetaServiceError::CommonError(format!(
            "Column family {} not found",
            DB_COLUMN_FAMILY_META_DATA
        ))
    })?;

    let file = std::fs::File::create(&dumping_path)?;
    let mut encoder = zstd::Encoder::new(file, 3)?;

    let iterator = snapshot_db.iterator_cf(
        &cf_handle,
        IteratorMode::From(prefix, rocksdb::Direction::Forward),
    );
    for item in iterator {
        let (key, value) = item?;
        if !key.starts_with(prefix) {
            break;
        }
        encoder.write_all(&(key.len() as u32).to_le_bytes())?;
        encoder.write_all(&key)?;
        encoder.write_all(&(value.len() as u32).to_le_bytes())?;
        encoder.write_all(&value)?;
    }

    encoder.finish()?;
    std::fs::rename(&dumping_path, snapshot_name)?;

    Ok(())
}

pub fn cleanup_old_snapshots(machine: &str) -> std::io::Result<()> {
    let conf = broker_config();
    let snapshot_dir = storage_raft_snapshot_fold(&conf.rocksdb.data_path);

    let entries = match std::fs::read_dir(&snapshot_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            info!(
                "[{}] Snapshot directory not found, skipping cleanup",
                machine
            );
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let prefix = format!("{}-", machine);
    let mut snapshots = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with(&prefix) && file_name.ends_with(".bin") {
                snapshots.push(file_name.to_string());
            }
        }
    }

    snapshots.sort();
    snapshots.reverse();

    if snapshots.len() > 5 {
        let to_delete = &snapshots[5..];
        info!(
            "[{}] Cleaning up {} old snapshots, keeping {} recent ones",
            machine,
            to_delete.len(),
            5
        );

        for snapshot_file in to_delete {
            let snapshot_id = snapshot_file.trim_end_matches(".bin");

            let bin_path = format!("{}/{}", snapshot_dir, snapshot_file);
            let meta_path = format!("{}/{}.meta", snapshot_dir, snapshot_id);

            if let Err(e) = std::fs::remove_file(&bin_path) {
                error!(
                    "[{}] Failed to delete snapshot file {}: {}",
                    machine, bin_path, e
                );
            } else {
                info!("[{}] Deleted snapshot file: {}", machine, bin_path);
            }

            if let Err(e) = std::fs::remove_file(&meta_path) {
                if e.kind() != ErrorKind::NotFound {
                    error!(
                        "[{}] Failed to delete meta file {}: {}",
                        machine, meta_path, e
                    );
                }
            } else {
                info!("[{}] Deleted meta file: {}", machine, meta_path);
            }
        }
    } else {
        info!(
            "[{}] Has {} snapshots, no cleanup needed (keeping all)",
            machine,
            snapshots.len()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{save_last_snapshot_id_to_path, save_snapshot_meta_to_path};
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

        save_last_snapshot_id_to_path(machine, snapshot_id_2, Some(&test_dir))
            .await
            .unwrap();

        std::fs::remove_dir_all(&test_dir).ok();
    }
}

