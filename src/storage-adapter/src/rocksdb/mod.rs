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

use crate::storage::{ShardConfig, ShardOffset, StorageAdapter};

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
    pub fn new(db_path: &str, max_open_files: i32) -> Self {
        RocksDBStorageAdapter {
            db: Arc::new(RocksDBEngine::new(
                db_path,
                max_open_files,
                column_family_list(),
            )),
            write_handles: DashMap::with_capacity(2),
        }
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
}

impl RocksDBStorageAdapter {
    async fn handle_write_request(
        &self,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let write_handle = self.get_write_handle().await;

        let (resp_sx, resp_rx) = oneshot::channel();

        let data = WriteThreadData::new(namespace, shard_name, messages, resp_sx);

        write_handle.data_sender.send(data).await.map_err(|err| {
            CommonError::CommonError(format!("Failed to send data to write thread: {}", err))
        })?;

        let time_res: Result<Vec<u64>, CommonError> = timeout(Duration::from_secs(30), resp_rx)
            .await
            .map_err(|err| {
                CommonError::CommonError(format!("Timeout while waiting for response: {}", err))
            })?
            .map_err(|err| {
                CommonError::CommonError(format!("Failed to receive response: {}", err))
            })?;

        Ok(time_res?)
    }

    async fn get_write_handle(&self) -> ThreadWriteHandle {
        let handle_key = "write_handle".to_string();
        if !self.write_handles.contains_key(&handle_key) {
            self.create_write_thread().await;
        }
        self.write_handles.get(&handle_key).unwrap().clone() // unwrap is safe here because we created it right before
    }

    async fn register_write_handle(&self, handle: ThreadWriteHandle) {
        let handle_key = "write_handle".to_string();
        self.write_handles.insert(handle_key, handle);
    }

    async fn create_write_thread(&self) {
        let (data_sender, data_recv) = mpsc::channel::<WriteThreadData>(1000);
        let (stop_sender, stop_recv) = broadcast::channel::<bool>(1);

        Self::spawn_write_thread(self.db.clone(), stop_recv, data_recv).await;

        let write_handle = ThreadWriteHandle {
            data_sender,
            stop_sender,
        };

        self.register_write_handle(write_handle).await;
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
                            CommonError::CommonError(format!("Failed to send response in write thread"))
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
    async fn create_shard(
        &self,
        namespace: String,
        shard_name: String,
        _: ShardConfig,
    ) -> Result<(), CommonError> {
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

        self.db.write(cf, shard_offset_key.as_str(), &0_u64)
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let shard_offset_key = Self::shard_offset_key(&namespace, &shard_name);

        // check whether the shard exists
        if self
            .db
            .read::<u64>(cf.clone(), shard_offset_key.as_str())?
            .is_none()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} under namespace {} not exists",
                shard_name, namespace
            )));
        }

        self.db.delete(cf, &shard_offset_key)
    }

    async fn write(
        &self,
        namespace: String,
        shard_name: String,
        message: Record,
    ) -> Result<u64, CommonError> {
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
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let shard_record_key_prefix = Self::shard_record_key_prefix(&namespace, &shard_name);

        let records = self
            .db
            .read_prefix(cf.clone(), &shard_record_key_prefix)?
            .into_iter()
            .map(|(_, v)| serde_json::from_slice::<Record>(&v).unwrap()) // safe to unwrap here because the data was serialized correctly before writing to the database
            .skip_while(|r| r.offset.unwrap() < offset) // safe to unwrap here because we set the offset to the record before writing to the database
            .take(read_config.max_record_num as usize)
            .collect::<Vec<_>>();

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
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let tag_offset_key_preix = Self::tag_offsets_key_prefix(&namespace, &shard_name, &tag);

        let offsets = self
            .db
            .read_prefix(cf.clone(), &tag_offset_key_preix)?
            .into_iter()
            .map(|(_, v)| serde_json::from_slice::<u64>(&v).unwrap())
            .skip_while(|r| *r < offset)
            .take(read_config.max_record_num as usize)
            .collect::<Vec<_>>();

        let mut records = Vec::new();

        for offset in offsets {
            let shard_record_key = Self::shard_record_key(&namespace, &shard_name, offset);
            let record = self
                .db
                .read::<Record>(cf.clone(), &shard_record_key)?
                .unwrap(); // safe to unwrap here since the record corresponding to the offset is guaranteed to exist
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
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let key_offset_key = Self::key_offset_key(&namespace, &shard_name, &key);

        match self.db.read::<u64>(cf.clone(), &key_offset_key)? {
            Some(key_offset) if key_offset >= offset && read_config.max_record_num >= 1 => {
                let shard_record_key = Self::shard_record_key(&namespace, &shard_name, offset);
                let record = self
                    .db
                    .read::<Record>(cf.clone(), &shard_record_key)?
                    .unwrap();

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
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let shard_record_key_prefix = Self::shard_record_key_prefix(&namespace, &shard_name);

        let res = self
            .db
            .read_prefix(cf.clone(), &shard_record_key_prefix)?
            .into_iter()
            .map(|(_, v)| serde_json::from_slice::<Record>(&v).unwrap())
            .find(|r| r.timestamp >= timestamp)
            .map(|r| ShardOffset {
                offset: r.offset.unwrap(),
                ..Default::default()
            });

        Ok(res)
    }

    async fn get_offset_by_group(
        &self,
        group_name: String,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let group_record_offsets_key_prefix = Self::group_record_offsets_key_prefix(&group_name);

        let offsets = self
            .db
            .read_prefix(cf.clone(), &group_record_offsets_key_prefix)?
            .into_iter()
            .map(|(_, v)| {
                let offset = serde_json::from_slice::<u64>(&v).unwrap();
                ShardOffset {
                    offset,
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();

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
        let write_handle = self.get_write_handle().await;

        write_handle
            .stop_sender
            .send(true)
            .map_err(CommonError::TokioBroadcastSendErrorBool)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use common_base::tools::unique_id;
    use metadata_struct::adapter::{read_config::ReadConfig, record::Record};

    use crate::storage::{ShardConfig, StorageAdapter};

    use super::RocksDBStorageAdapter;
    #[tokio::test]
    async fn stream_read_write() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());

        let storage_adapter = RocksDBStorageAdapter::new(db_path.as_str(), 100);
        let namespace = unique_id();
        let shard_name = "test-11".to_string();

        // step 1: create shard
        storage_adapter
            .create_shard(
                namespace.clone(),
                shard_name.clone(),
                ShardConfig::default(),
            )
            .await
            .unwrap();

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
                        max_record_num: 10,
                        max_size: 1024,
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
                    max_record_num: 10,
                    max_size: 1024,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert_eq!(result_read.len(), 2);

        // test group functionalities
        let group_id = "test_group_id".to_string();
        let read_config = ReadConfig {
            max_record_num: 1,
            ..Default::default()
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
            .delete_shard(namespace, shard_name)
            .await
            .unwrap();

        let _ = std::fs::remove_dir_all(&db_path);
    }
}
