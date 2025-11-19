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

use super::{get_last_snapshot_id, get_snapshot_meta, snapshot_name};
use crate::raft::manager::RaftStateMachineName;
use crate::raft::type_config::{StorageResult, TypeConfig};
use openraft::{Snapshot, StorageError};
use rocksdb::DB;
use rocksdb_engine::storage::family::{DB_COLUMN_FAMILY_META_DATA, DB_COLUMN_FAMILY_META_METADATA};
use std::io::{ErrorKind, Read};
use std::sync::Arc;
use tokio::fs::File;
use tracing::{error, info};

pub async fn recover_snapshot(
    machine: &RaftStateMachineName,
    db: &Arc<DB>,
    snapshot: Snapshot<TypeConfig>,
) -> StorageResult<()> {
    let row_db = db.clone();
    let row_machine = machine.clone();
    let snapshot_id = snapshot.meta.snapshot_id.clone();

    info!(
        "[{}] Starting to recover snapshot, snapshot_id={}",
        machine, snapshot_id
    );

    tokio::spawn(async move {
        let res = match row_machine {
            RaftStateMachineName::METADATA => recover_snapshot_by_metadata(&row_db, snapshot).await,
            RaftStateMachineName::OFFSET => recover_snapshot_by_offset(&row_db, snapshot).await,
            RaftStateMachineName::MQTT => recover_snapshot_by_mqtt(&row_db, snapshot).await,
        };
        if let Err(e) = res {
            error!(
                "[{}] Failed to recover snapshot from snapshot_id={}: {:?}",
                row_machine, snapshot_id, e
            );
        } else {
            info!(
                "[{}] Snapshot recovery completed successfully for snapshot_id={}",
                row_machine, snapshot_id
            );
        }
    });

    Ok(())
}

async fn recover_snapshot_by_metadata(
    db: &Arc<DB>,
    snapshot: Snapshot<TypeConfig>,
) -> StorageResult<()> {
    let snapshot_file = snapshot.snapshot.into_std().await;

    let cf_handle = db
        .cf_handle(DB_COLUMN_FAMILY_META_METADATA)
        .ok_or_else(|| {
            StorageError::read(&std::io::Error::new(
                ErrorKind::NotFound,
                format!("Column family {} not found", DB_COLUMN_FAMILY_META_METADATA),
            ))
        })?;

    let mut decoder = zstd::Decoder::new(snapshot_file)
        .map_err(|e| StorageError::read(&std::io::Error::new(ErrorKind::InvalidData, e)))?;

    let mut batch = rocksdb::WriteBatch::default();
    let mut count = 0u64;
    let mut len_buf = [0u8; 4];

    loop {
        match decoder.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(StorageError::read(&e)),
        }
        let key_len = u32::from_le_bytes(len_buf) as usize;

        let mut key = vec![0u8; key_len];
        decoder
            .read_exact(&mut key)
            .map_err(|e| StorageError::read(&e))?;

        decoder
            .read_exact(&mut len_buf)
            .map_err(|e| StorageError::read(&e))?;
        let value_len = u32::from_le_bytes(len_buf) as usize;

        let mut value = vec![0u8; value_len];
        decoder
            .read_exact(&mut value)
            .map_err(|e| StorageError::read(&e))?;

        batch.put_cf(&cf_handle, key, value);
        count += 1;

        if count.is_multiple_of(1000) {
            db.write(batch).map_err(|e| StorageError::write(&e))?;
            batch = rocksdb::WriteBatch::default();
            info!("[metadata] Recovered {} entries so far...", count);
        }
    }

    if !batch.is_empty() {
        db.write(batch).map_err(|e| StorageError::write(&e))?;
    }

    info!(
        "[metadata] Successfully recovered {} entries from snapshot",
        count
    );

    Ok(())
}

async fn recover_snapshot_by_mqtt(
    db: &Arc<DB>,
    snapshot: Snapshot<TypeConfig>,
) -> StorageResult<()> {
    let snapshot_file = snapshot.snapshot.into_std().await;

    let cf_handle = db.cf_handle(DB_COLUMN_FAMILY_META_DATA).ok_or_else(|| {
        StorageError::read(&std::io::Error::new(
            ErrorKind::NotFound,
            format!("Column family {} not found", DB_COLUMN_FAMILY_META_DATA),
        ))
    })?;

    let mut decoder = zstd::Decoder::new(snapshot_file)
        .map_err(|e| StorageError::read(&std::io::Error::new(ErrorKind::InvalidData, e)))?;

    let mut batch = rocksdb::WriteBatch::default();
    let mut count = 0u64;
    let mut len_buf = [0u8; 4];

    loop {
        match decoder.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(StorageError::read(&e)),
        }
        let key_len = u32::from_le_bytes(len_buf) as usize;

        let mut key = vec![0u8; key_len];
        decoder
            .read_exact(&mut key)
            .map_err(|e| StorageError::read(&e))?;

        decoder
            .read_exact(&mut len_buf)
            .map_err(|e| StorageError::read(&e))?;
        let value_len = u32::from_le_bytes(len_buf) as usize;

        let mut value = vec![0u8; value_len];
        decoder
            .read_exact(&mut value)
            .map_err(|e| StorageError::read(&e))?;

        batch.put_cf(&cf_handle, key, value);
        count += 1;

        if count.is_multiple_of(1000) {
            db.write(batch).map_err(|e| StorageError::write(&e))?;
            batch = rocksdb::WriteBatch::default();
            info!("[mqtt] Recovered {} entries so far...", count);
        }
    }

    if !batch.is_empty() {
        db.write(batch).map_err(|e| StorageError::write(&e))?;
    }

    info!(
        "[mqtt] Successfully recovered {} entries from snapshot",
        count
    );

    Ok(())
}

async fn recover_snapshot_by_offset(
    db: &Arc<DB>,
    snapshot: Snapshot<TypeConfig>,
) -> StorageResult<()> {
    let snapshot_file = snapshot.snapshot.into_std().await;

    let cf_handle = db.cf_handle(DB_COLUMN_FAMILY_META_DATA).ok_or_else(|| {
        StorageError::read(&std::io::Error::new(
            ErrorKind::NotFound,
            format!("Column family {} not found", DB_COLUMN_FAMILY_META_DATA),
        ))
    })?;

    let mut decoder = zstd::Decoder::new(snapshot_file)
        .map_err(|e| StorageError::read(&std::io::Error::new(ErrorKind::InvalidData, e)))?;

    let mut batch = rocksdb::WriteBatch::default();
    let mut count = 0u64;
    let mut len_buf = [0u8; 4];

    loop {
        match decoder.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(StorageError::read(&e)),
        }
        let key_len = u32::from_le_bytes(len_buf) as usize;

        let mut key = vec![0u8; key_len];
        decoder
            .read_exact(&mut key)
            .map_err(|e| StorageError::read(&e))?;

        decoder
            .read_exact(&mut len_buf)
            .map_err(|e| StorageError::read(&e))?;
        let value_len = u32::from_le_bytes(len_buf) as usize;

        let mut value = vec![0u8; value_len];
        decoder
            .read_exact(&mut value)
            .map_err(|e| StorageError::read(&e))?;

        batch.put_cf(&cf_handle, key, value);
        count += 1;

        if count.is_multiple_of(1000) {
            db.write(batch).map_err(|e| StorageError::write(&e))?;
            batch = rocksdb::WriteBatch::default();
            info!("[offset] Recovered {} entries so far...", count);
        }
    }

    if !batch.is_empty() {
        db.write(batch).map_err(|e| StorageError::write(&e))?;
    }

    info!(
        "[offset] Successfully recovered {} entries from snapshot",
        count
    );

    Ok(())
}

pub async fn get_current_snapshot(machine: &str) -> StorageResult<Option<Snapshot<TypeConfig>>> {
    let snapshot_id = match get_last_snapshot_id(machine).await {
        Ok(id) => id,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            info!("[{}] No snapshot found, returning None", machine);
            return Ok(None);
        }
        Err(e) => return Err(StorageError::read(&e)),
    };

    let snapshot_meta = match get_snapshot_meta(&snapshot_id).await {
        Ok(meta) => meta,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            info!(
                "[{}] Snapshot metadata not found for snapshot_id={}, returning None",
                machine, snapshot_id
            );
            return Ok(None);
        }
        Err(e) => return Err(StorageError::read(&e)),
    };

    let res = match File::open(&snapshot_name(&snapshot_id)).await {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            info!(
                "[{}] Snapshot file not found for snapshot_id={}, returning None",
                machine, snapshot_id
            );
            return Ok(None);
        }
        Err(e) => return Err(StorageError::read(&e)),
    };

    Ok(Some(Snapshot {
        meta: snapshot_meta,
        snapshot: res,
    }))
}

#[cfg(test)]
mod tests {
    use super::super::{get_last_snapshot_id_from_path, get_snapshot_meta_from_path};
    use crate::raft::type_config::{Node, TypeConfig};
    use common_base::utils::file_utils::test_temp_dir;
    use openraft::vote::leader_id_adv::LeaderId;
    use openraft::{LogId, Membership, SnapshotMeta, StoredMembership};
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs::create_dir_all;

    #[tokio::test]
    async fn test_get_snapshot_meta() {
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

        let _meta = SnapshotMeta::<TypeConfig> {
            last_log_id: Some(log_id),
            last_membership: stored_membership.clone(),
            snapshot_id: snapshot_id.clone(),
        };

        assert!(get_snapshot_meta_from_path("non-existent", Some(&test_dir))
            .await
            .is_err());

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[tokio::test]
    async fn test_get_last_snapshot_id() {
        let test_dir = format!("{}/last_snapshot_id_test", test_temp_dir());
        create_dir_all(&test_dir).unwrap();

        assert!(
            get_last_snapshot_id_from_path("non-existent", Some(&test_dir))
                .await
                .is_err()
        );

        std::fs::remove_dir_all(&test_dir).ok();
    }
}
