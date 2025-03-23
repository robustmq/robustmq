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

use std::{collections::HashMap, fmt::Display, sync::Arc, time::Duration};

use axum::async_trait;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use rocksdb_engine::RocksDBEngine;
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{self, Receiver},
        oneshot,
    },
    time::{sleep, timeout},
};

use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};

const DB_COLUMN_FAMILY: &str = "db";

fn column_family_list() -> Vec<String> {
    vec![DB_COLUMN_FAMILY.to_string()]
}

pub struct RocksDBStorageAdapter {
    pub db: Arc<RocksDBEngine>,
    write_handles: DashMap<String, ThreadWriteHandle>,
}

struct WriteThreadData {
    namespace: String,
    shard: String,
    records: Vec<Record>,
    resp_sx: oneshot::Sender<Result<Vec<u64>, CommonError>>, // thread response: offset or error
}

#[derive(Clone)]
struct ThreadWriteHandle {
    data_sender: mpsc::Sender<WriteThreadData>,
    stop_sender: broadcast::Sender<bool>,
}

impl WriteThreadData {
    fn new(
        namespace: String,
        shard: String,
        records: Vec<Record>,
        resp_sx: oneshot::Sender<Result<Vec<u64>, CommonError>>,
    ) -> Self {
        WriteThreadData {
            namespace,
            shard,
            records,
            resp_sx,
        }
    }
}

impl RocksDBStorageAdapter {
    pub fn new(db_path: impl AsRef<str>, max_open_files: i32) -> Self {
        RocksDBStorageAdapter {
            db: Arc::new(RocksDBEngine::new(
                db_path.as_ref(),
                max_open_files,
                column_family_list(),
            )),
            write_handles: DashMap::with_capacity(2),
        }
    }

    pub fn ensure_shard_exists(
        &self,
        namespace: impl AsRef<str>,
        shard: impl AsRef<str>,
    ) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();
        let shard_offset_key = Self::shard_offset_key(&namespace.as_ref(), &shard.as_ref());

        if self
            .db
            .read::<u64>(cf.clone(), shard_offset_key.as_str())?
            .is_none()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} under namespace {} not exists",
                shard.as_ref(),
                namespace.as_ref()
            )));
        }

        Ok(())
    }

    #[inline(always)]
    pub fn shard_record_key<S1: Display>(namespace: &S1, shard: &S1, record_offset: u64) -> String {
        format!(
            "/record/{}/{}/record/{:020}",
            namespace, shard, record_offset
        )
    }

    #[inline(always)]
    pub fn shard_record_key_prefix<S1: Display>(namespace: &S1, shard: &S1) -> String {
        format!("/record/{}/{}/record/", namespace, shard)
    }

    #[inline(always)]
    pub fn shard_offset_key<S1: Display>(namespace: &S1, shard: &S1) -> String {
        format!("/offset/{}/{}", namespace, shard)
    }

    #[inline(always)]
    pub fn key_offset_key<S1: Display>(namespace: &S1, shard: &S1, key: &S1) -> String {
        format!("/key/{}/{}/{}", namespace, shard, key)
    }

    #[inline(always)]
    pub fn tag_offsets_key<S1: Display>(
        namespace: &S1,
        shard: &S1,
        tag: &S1,
        offset: u64,
    ) -> String {
        format!("/tag/{}/{}/{}/{:020}", namespace, shard, tag, offset)
    }

    #[inline(always)]
    pub fn tag_offsets_key_prefix<S1: Display>(namespace: &S1, shard: &S1, tag: &S1) -> String {
        format!("/tag/{}/{}/{}/", namespace, shard, tag)
    }

    #[inline(always)]
    pub fn group_record_offsets_key<S1: Display>(group: &S1, namespace: &S1, shard: &S1) -> String {
        format!("/group/{}/{}/{}", group, namespace, shard)
    }

    #[inline(always)]
    pub fn group_record_offsets_key_prefix<S1: Display>(group: &S1) -> String {
        format!("/group/{}/", group)
    }

    #[inline(always)]
    pub fn shard_info_key<S1: Display>(namespace: &S1, shard: &S1) -> String {
        format!("/shard/{}/{}", namespace, shard)
    }
}

impl RocksDBStorageAdapter {
    #[inline(always)]
    fn write_handle_key(namespace: impl AsRef<str>, shard_name: impl AsRef<str>) -> String {
        format!("{}-{}", namespace.as_ref(), shard_name.as_ref())
    }

    async fn handle_write_request(
        &self,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let write_handle = self.get_write_handle(&namespace, &shard_name).await;

        let (resp_sx, resp_rx) = oneshot::channel();

        let data = WriteThreadData::new(namespace, shard_name, messages, resp_sx);

        write_handle.data_sender.send(data).await.map_err(|err| {
            CommonError::CommonError(format!("Failed to send data to write thread: {}", err))
        })?;

        timeout(Duration::from_secs(30), resp_rx)
            .await
            .map_err(|err| {
                CommonError::CommonError(format!("Timeout while waiting for response: {}", err))
            })?
            .map_err(|err| {
                CommonError::CommonError(format!("Failed to receive response: {}", err))
            })?
    }

    async fn get_write_handle(
        &self,
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
    ) -> ThreadWriteHandle {
        let handle_key = Self::write_handle_key(namespace.as_ref(), shard_name.as_ref());

        if !self.write_handles.contains_key(&handle_key) {
            self.create_write_thread(namespace.as_ref(), shard_name.as_ref())
                .await;
        }

        self.write_handles.get(&handle_key).unwrap().clone()
    }

    async fn get_all_write_handles(&self) -> Vec<ThreadWriteHandle> {
        self.write_handles
            .iter()
            .map(|item| item.value().clone())
            .collect()
    }

    async fn register_write_handle(
        &self,
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
        handle: ThreadWriteHandle,
    ) {
        let handle_key = Self::write_handle_key(namespace, shard_name);
        self.write_handles.insert(handle_key, handle);
    }

    async fn create_write_thread(&self, namespace: impl AsRef<str>, shard_name: impl AsRef<str>) {
        let (data_sender, data_recv) = mpsc::channel::<WriteThreadData>(1000);
        let (stop_sender, stop_recv) = broadcast::channel::<bool>(1);

        Self::spawn_write_thread(self.db.clone(), stop_recv, data_recv).await;

        let write_handle = ThreadWriteHandle {
            data_sender,
            stop_sender,
        };

        self.register_write_handle(namespace.as_ref(), shard_name.as_ref(), write_handle)
            .await;
    }

    async fn spawn_write_thread(
        db: Arc<RocksDBEngine>,
        mut stop_recv: broadcast::Receiver<bool>,
        mut data_recv: Receiver<WriteThreadData>,
    ) {
        tokio::spawn(async move {
            loop {
                select! {
                    val = stop_recv.recv() => {
                        if let Ok(flag) = val {
                            if flag {
                                break
                            }
                        }
                    },
                    val = data_recv.recv() => {
                        if val.is_none() {
                            sleep(Duration::from_millis(100)).await;
                            continue
                        }

                        let packet = val.unwrap();  // unwrap is safe here since we checked for None before
                        let res = Self::
                            thread_batch_write(db.clone(), packet.namespace, packet.shard, packet.records)
                            .await;

                        packet.resp_sx.send(res).map_err(|_| {
                            CommonError::CommonError("Failed to send response in write thread".to_string())
                        })?;

                    }
                }
            }

            Ok::<(), CommonError>(())
        });
    }

    async fn thread_batch_write(
        db: Arc<RocksDBEngine>,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let cf = db.cf_handle(DB_COLUMN_FAMILY).unwrap(); // unwrap is safe since we created this column family

        // get the starting shard offset
        let shard_offset_key = Self::shard_offset_key(&namespace, &shard_name);
        let offset = match db.read::<u64>(cf.clone(), shard_offset_key.as_str())? {
            Some(offset) => offset,
            None => {
                return Err(CommonError::CommonError(format!(
                    "shard {} under {} not exists",
                    shard_name, namespace
                )));
            }
        };

        let mut start_offset = offset;

        let mut offset_res = Vec::new();

        for mut msg in messages {
            offset_res.push(start_offset);
            msg.offset = Some(start_offset);

            // write the shard record
            let shard_record_key = Self::shard_record_key(&namespace, &shard_name, start_offset);
            db.write(cf.clone(), &shard_record_key, &msg)?;

            // write the key offset
            if !msg.key.is_empty() {
                let key_offset_key = Self::key_offset_key(&namespace, &shard_name, &msg.key);
                db.write(cf.clone(), key_offset_key.as_str(), &start_offset)?;
            }

            for tag in msg.tags.iter() {
                let tag_offsets_key =
                    Self::tag_offsets_key(&namespace, &shard_name, tag, start_offset);
                db.write(cf.clone(), &tag_offsets_key, &start_offset)?;
            }

            start_offset += 1;
        }

        // update the shard offset
        db.write(cf.clone(), shard_offset_key.as_str(), &start_offset)?;

        Ok(offset_res)
    }
}

#[async_trait]
impl StorageAdapter for RocksDBStorageAdapter {
    /// create a shard by inserting an offset 0
    async fn create_shard(&self, shard: ShardInfo) -> Result<(), CommonError> {
        let namespace = shard.namespace.clone();
        let shard_name = shard.shard_name.clone();

        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let shard_offset_key = Self::shard_offset_key(&namespace, &shard_name);

        // check whether the shard exists
        if self
            .db
            .read::<u64>(cf.clone(), shard_offset_key.as_str())?
            .is_some()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} under namespace {} already exists",
                shard_name, namespace
            )));
        }

        self.db
            .write(cf.clone(), shard_offset_key.as_str(), &0_u64)?;

        // store shard config
        self.db.write(
            cf,
            Self::shard_info_key(&namespace, &shard_name).as_str(),
            &shard,
        )
    }

    async fn list_shard(
        &self,
        namespace: String,
        shard_name: String,
    ) -> Result<Vec<ShardInfo>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let prefix_key = if namespace.is_empty() {
            "/shard/".to_string()
        } else {
            Self::shard_info_key(&namespace, &shard_name)
        };

        let raw_shard_info = self.db.read_prefix(cf.clone(), &prefix_key)?;

        let mut res = Vec::new();

        for (_, v) in raw_shard_info {
            let shard_info = serde_json::from_slice::<ShardInfo>(v.as_slice())?;
            res.push(shard_info);
        }

        Ok(res)
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        self.ensure_shard_exists(&namespace, &shard_name)?;

        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        self.db
            .delete(cf.clone(), &Self::shard_offset_key(&namespace, &shard_name))?;

        // also delete the shard info
        self.db
            .delete(cf, &Self::shard_info_key(&namespace, &shard_name))
    }

    async fn write(
        &self,
        namespace: String,
        shard_name: String,
        message: Record,
    ) -> Result<u64, CommonError> {
        self.ensure_shard_exists(&namespace, &shard_name)?;

        self.handle_write_request(namespace, shard_name, vec![message])
            .await
            .map(|offsets| offsets.first().cloned().unwrap()) // unwrap is safe here because we know the vector is not empty
    }

    async fn batch_write(
        &self,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        self.ensure_shard_exists(&namespace, &shard_name)?;

        self.handle_write_request(namespace, shard_name, messages)
            .await
    }

    async fn read_by_offset(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        self.ensure_shard_exists(&namespace, &shard_name)?;

        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let mut records = Vec::new();

        let mut total_size = 0;

        for i in offset..offset.saturating_add(read_config.max_record_num) {
            let shard_record_key = Self::shard_record_key(&namespace, &shard_name, i);
            let record = self.db.read::<Record>(cf.clone(), &shard_record_key)?;

            if record.is_none() {
                break;
            }

            let record_bytes = record.as_ref().unwrap().data.len() as u64;

            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record.unwrap());
        }

        Ok(records)
    }

    async fn read_by_tag(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        tag: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        self.ensure_shard_exists(&namespace, &shard_name)?;

        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let tag_offset_key_preix = Self::tag_offsets_key_prefix(&namespace, &shard_name, &tag);

        let raw_offsets = self.db.read_prefix(cf.clone(), &tag_offset_key_preix)?;

        let mut offsets = Vec::new();

        for (_, v) in raw_offsets {
            let record_offset = serde_json::from_slice::<u64>(&v)?;

            if record_offset >= offset && offsets.len() < read_config.max_record_num as usize {
                offsets.push(record_offset);
            }
        }

        let mut records = Vec::new();
        let mut total_size = 0;

        for offset in offsets {
            let shard_record_key = Self::shard_record_key(&namespace, &shard_name, offset);
            let record = self
                .db
                .read::<Record>(cf.clone(), &shard_record_key)?
                .ok_or(CommonError::CommonError("Record not found".to_string()))?;

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record);
        }

        Ok(records)
    }

    async fn read_by_key(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        key: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        self.ensure_shard_exists(&namespace, &shard_name)?;

        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let key_offset_key = Self::key_offset_key(&namespace, &shard_name, &key);

        match self.db.read::<u64>(cf.clone(), &key_offset_key)? {
            Some(key_offset) if key_offset >= offset && read_config.max_record_num >= 1 => {
                let shard_record_key = Self::shard_record_key(&namespace, &shard_name, offset);
                let record = self
                    .db
                    .read::<Record>(cf.clone(), &shard_record_key)?
                    .ok_or(CommonError::CommonError("Record not found".to_string()))?;

                if record.data.len() as u64 > read_config.max_size {
                    return Ok(Vec::new());
                }

                return Ok(vec![record]);
            }
            _ => return Ok(Vec::new()),
        };
    }

    async fn get_offset_by_timestamp(
        &self,
        namespace: String,
        shard_name: String,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        self.ensure_shard_exists(&namespace, &shard_name)?;

        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let shard_record_key_prefix = Self::shard_record_key_prefix(&namespace, &shard_name);

        let raw_res = self.db.read_prefix(cf.clone(), &shard_record_key_prefix)?;

        for (_, v) in raw_res {
            let record = serde_json::from_slice::<Record>(&v)?;

            if record.timestamp >= timestamp {
                return Ok(Some(ShardOffset {
                    offset: record.offset.ok_or(CommonError::CommonError(
                        "Record offset is None".to_string(),
                    ))?,
                    ..Default::default()
                }));
            }
        }

        Ok(None)
    }

    async fn get_offset_by_group(
        &self,
        group_name: String,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let group_record_offsets_key_prefix = Self::group_record_offsets_key_prefix(&group_name);

        let raw_offsets = self
            .db
            .read_prefix(cf.clone(), &group_record_offsets_key_prefix)?;

        let mut offsets = Vec::new();

        for (_, v) in raw_offsets {
            let offset = serde_json::from_slice::<u64>(&v)?;

            offsets.push(ShardOffset {
                offset,
                ..Default::default()
            });
        }

        Ok(offsets)
    }

    async fn commit_offset(
        &self,
        group_name: String,
        namespace: String,
        offsets: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        offsets.into_iter().try_for_each(|(shard_name, offset)| {
            let group_record_offsets_key =
                Self::group_record_offsets_key(&group_name, &namespace, &shard_name);

            self.db
                .write(cf.clone(), &group_record_offsets_key, &offset)
        })?;

        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        let write_handles = self.get_all_write_handles().await;

        for handle in write_handles {
            handle
                .stop_sender
                .send(true)
                .map_err(CommonError::TokioBroadcastSendErrorBool)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc, vec};

    use common_base::{tools::unique_id, utils::crc::calc_crc32};
    use futures::future;
    use metadata_struct::adapter::{
        read_config::ReadConfig,
        record::{Header, Record},
    };

    use crate::storage::{ShardInfo, StorageAdapter};

    use super::RocksDBStorageAdapter;
    #[tokio::test]
    async fn stream_read_write() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());

        let storage_adapter = RocksDBStorageAdapter::new(db_path.as_str(), 100);
        let namespace = unique_id();
        let shard_name = "test-11".to_string();

        // step 1: create shard
        storage_adapter
            .create_shard(ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        // step 2: list the shard just created
        let shards = storage_adapter
            .list_shard(namespace.clone(), shard_name.clone())
            .await
            .unwrap();

        assert_eq!(shards.len(), 1);
        assert_eq!(shards.first().unwrap().shard_name, shard_name);
        assert_eq!(shards.first().unwrap().namespace, namespace);
        assert_eq!(shards.first().unwrap().replica_num, 1);

        // insert two records (no key or tag) into the shard
        let ms1 = "test1".to_string();
        let ms2 = "test2".to_string();
        let data = vec![
            Record::build_byte(ms1.clone().as_bytes().to_vec()),
            Record::build_byte(ms2.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);

        // read previous records
        assert_eq!(
            storage_adapter
                .read_by_offset(
                    namespace.clone(),
                    shard_name.clone(),
                    0,
                    ReadConfig {
                        max_record_num: u64::MAX,
                        max_size: u64::MAX,
                    }
                )
                .await
                .unwrap()
                .len(),
            2
        );

        // insert two other records (no key or tag) into the shard
        let ms3 = "test3".to_string();
        let ms4 = "test4".to_string();
        let data = vec![
            Record::build_byte(ms3.clone().as_bytes().to_vec()),
            Record::build_byte(ms4.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();

        // read from offset 2
        let result_read = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                2,
                ReadConfig {
                    max_record_num: u64::MAX,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert_eq!(result_read.len(), 2);

        // test group functionalities
        let group_id = unique_id();
        let read_config = ReadConfig {
            max_record_num: 1,
            max_size: u64::MAX,
        };

        // read m1
        let offset = 0;
        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset,
                read_config.clone(),
            )
            .await
            .unwrap();

        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms1
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );

        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read ms2
        let offset = storage_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset.first().unwrap().offset + 1,
                read_config.clone(),
            )
            .await
            .unwrap();

        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms2
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );
        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read m3
        let offset: Vec<crate::storage::ShardOffset> = storage_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset.first().unwrap().offset + 1,
                read_config.clone(),
            )
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms3
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );
        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read m4
        let offset = storage_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset.first().unwrap().offset + 1,
                read_config.clone(),
            )
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms4
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );
        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // delete shard
        storage_adapter
            .delete_shard(namespace.clone(), shard_name.clone())
            .await
            .unwrap();

        // check if the shard is deleted
        let shards = storage_adapter
            .list_shard(namespace, shard_name)
            .await
            .unwrap();

        assert_eq!(shards.len(), 0);

        storage_adapter.close().await.unwrap();

        let _ = std::fs::remove_dir_all(&db_path);
    }

    #[tokio::test]
    #[ignore]
    async fn concurrency_test() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());

        let storage_adapter = Arc::new(RocksDBStorageAdapter::new(db_path.as_str(), 100));

        // create one namespace with 4 shards
        let namespace = unique_id();
        let shards = (0..4).map(|i| format!("test-{}", i)).collect::<Vec<_>>();

        // create shards
        for i in 0..shards.len() {
            storage_adapter
                .create_shard(ShardInfo {
                    namespace: namespace.clone(),
                    shard_name: shards.get(i).unwrap().clone(),
                    replica_num: 1,
                })
                .await
                .unwrap();
        }

        // list the shard we just created
        let list_res = storage_adapter
            .list_shard(namespace.clone(), "".to_string())
            .await
            .unwrap();

        assert_eq!(list_res.len(), 4);

        let header = vec![Header {
            name: "name".to_string(),
            value: "value".to_string(),
        }];

        // create 10,000 tokio tasks, each of which will write 100 records to a shard
        let mut tasks = vec![];
        for tid in 0..10000 {
            let storage_adapter = storage_adapter.clone();
            let namespace = namespace.clone();
            let shard_name = shards.get(tid % shards.len()).unwrap().clone();
            let header = header.clone();

            let task = tokio::spawn(async move {
                let mut batch_data = Vec::new();

                for idx in 0..100 {
                    let value = format!("data-{}-{}", tid, idx).as_bytes().to_vec();
                    let data = Record {
                        offset: None,
                        header: header.clone(),
                        key: format!("key-{}-{}", tid, idx),
                        data: value.clone(),
                        tags: vec![format!("task-{}", tid)],
                        timestamp: 0,
                        delay_timestamp: 0,
                        crc_num: calc_crc32(&value),
                    };

                    batch_data.push(data);
                }

                let write_offsets = storage_adapter
                    .batch_write(namespace.clone(), shard_name.clone(), batch_data.clone())
                    .await
                    .unwrap();

                assert_eq!(write_offsets.len(), 100);

                let mut read_records = Vec::new();

                for offset in write_offsets.iter() {
                    let records = storage_adapter
                        .read_by_offset(
                            namespace.clone(),
                            shard_name.clone(),
                            *offset,
                            ReadConfig {
                                max_record_num: 1,
                                max_size: u64::MAX,
                            },
                        )
                        .await
                        .unwrap();

                    read_records.extend(records);
                }

                for (l, r) in batch_data.into_iter().zip(read_records.iter()) {
                    assert_eq!(l.tags, r.tags);
                    assert_eq!(l.key, r.key);
                    assert_eq!(l.data, r.data);
                }

                // test read by tag
                let tag_records = storage_adapter
                    .read_by_tag(
                        namespace.clone(),
                        shard_name.clone(),
                        0,
                        format!("task-{}", tid),
                        ReadConfig {
                            max_record_num: u64::MAX,
                            max_size: u64::MAX,
                        },
                    )
                    .await
                    .unwrap();

                assert_eq!(tag_records.len(), 100);

                for (l, r) in read_records.into_iter().zip(tag_records) {
                    assert_eq!(l.offset, r.offset);
                    assert_eq!(l.tags, r.tags);
                    assert_eq!(l.key, r.key);
                    assert_eq!(l.data, r.data);
                }
            });

            tasks.push(task);
        }

        future::join_all(tasks).await;

        for shard in shards.iter() {
            let len = storage_adapter
                .read_by_offset(
                    namespace.clone(),
                    shard.clone(),
                    0,
                    ReadConfig {
                        max_record_num: u64::MAX,
                        max_size: u64::MAX,
                    },
                )
                .await
                .unwrap()
                .len();

            assert_eq!(len, (10000 / shards.len()) * 100);
        }

        // delete all shards
        for shard in shards.iter() {
            storage_adapter
                .delete_shard(namespace.clone(), shard.clone())
                .await
                .unwrap();
        }

        // check if the shards are deleted
        let list_res = storage_adapter
            .list_shard(namespace.clone(), "".to_string())
            .await
            .unwrap();

        assert_eq!(list_res.len(), 0);

        storage_adapter.close().await.unwrap();

        let _ = std::fs::remove_dir_all(&db_path);
    }
}
