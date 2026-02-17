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

use crate::raft::type_config::{Node, NodeId};
use bincode::{deserialize, serialize};
use common_config::broker::broker_config;
use openraft::SnapshotMeta;
use rocksdb_engine::storage::family::storage_raft_snapshot_fold;
use std::io::ErrorKind;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub mod build;
pub mod recover;

pub(crate) async fn save_last_snapshot_id(
    machine: &str,
    last_snapshot_id: &str,
) -> std::io::Result<()> {
    save_last_snapshot_id_to_path(machine, last_snapshot_id, None).await
}

pub(crate) async fn get_last_snapshot_id(machine: &str) -> std::io::Result<String> {
    get_last_snapshot_id_from_path(machine, None).await
}

pub(crate) async fn save_last_snapshot_id_to_path(
    machine: &str,
    last_snapshot_id: &str,
    custom_path: Option<&str>,
) -> std::io::Result<()> {
    let file_path = if let Some(path) = custom_path {
        format!("{}/{}.last_snapshot_id", path, machine)
    } else {
        machine_last_snapshot_id(machine)
    };

    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(&file_path).await?;
    file.write_all(last_snapshot_id.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

pub(crate) async fn get_last_snapshot_id_from_path(
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

pub(crate) async fn save_snapshot_meta(meta: SnapshotMeta<NodeId, Node>) -> std::io::Result<()> {
    save_snapshot_meta_to_path(meta, None).await
}

pub(crate) async fn get_snapshot_meta(
    snapshot_id: &str,
) -> std::io::Result<SnapshotMeta<NodeId, Node>> {
    get_snapshot_meta_from_path(snapshot_id, None).await
}

pub(crate) async fn save_snapshot_meta_to_path(
    meta: SnapshotMeta<NodeId, Node>,
    custom_path: Option<&str>,
) -> std::io::Result<()> {
    let file_path = if let Some(path) = custom_path {
        format!("{}/{}.meta", path, meta.snapshot_id)
    } else {
        snapshot_meta(&meta.snapshot_id)
    };

    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let data = serialize(&meta).map_err(|e| {
        std::io::Error::new(ErrorKind::InvalidData, format!("Serialize error: {}", e))
    })?;

    let mut file = File::create(&file_path).await?;
    file.write_all(&data).await?;
    file.flush().await?;
    Ok(())
}

pub(crate) async fn get_snapshot_meta_from_path(
    snapshot_id: &str,
    custom_path: Option<&str>,
) -> std::io::Result<SnapshotMeta<NodeId, Node>> {
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

pub(crate) fn snapshot_name(snapshot_id: &str) -> String {
    let conf = broker_config();
    format!(
        "{}/{}.bin",
        storage_raft_snapshot_fold(&conf.rocksdb.data_path),
        snapshot_id
    )
}

pub(crate) fn snapshot_meta(snapshot_id: &str) -> String {
    let conf = broker_config();
    format!(
        "{}/{}.meta",
        storage_raft_snapshot_fold(&conf.rocksdb.data_path),
        snapshot_id
    )
}

pub(crate) fn machine_last_snapshot_id(machine: &str) -> String {
    let conf = broker_config();
    format!(
        "{}/{}.last_snapshot_id",
        storage_raft_snapshot_fold(&conf.rocksdb.data_path),
        machine
    )
}
