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

use std::{collections::HashMap, time::Duration};

use axum::async_trait;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use futures::TryStreamExt;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use opendal::{services::S3, EntryMode, Operator};
use serde::{Deserialize, Serialize};
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

#[derive(Debug)]
#[allow(dead_code)]
struct WriteThreadData {
    namespace: String,
    shard: String,
    records: Vec<Record>,
    resp_sx: oneshot::Sender<Result<Vec<u64>, CommonError>>, // thread response: offset or error
}

#[derive(Clone)]
#[allow(dead_code)]
struct ThreadWriteHandle {
    data_sender: mpsc::Sender<WriteThreadData>,
    stop_sender: broadcast::Sender<bool>,
}

#[derive(Serialize, Deserialize)]
struct ShardConfigStore {
    namespace: String,
    shard_name: String,
    shard_config: ShardConfig,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
struct MinIoStorageAdapter {
    op: Operator,
    write_handles: DashMap<String, ThreadWriteHandle>,
}

#[allow(dead_code)]
impl MinIoStorageAdapter {
    pub fn new(data_dir: impl AsRef<str>, bucket: impl AsRef<str>) -> Result<Self, CommonError> {
        let builder = S3::default()
            .root(data_dir.as_ref())
            .bucket(bucket.as_ref())
            .endpoint("http://127.0.0.1:9000")
            .access_key_id("minioadmin")
            .secret_access_key("minioadmin");
        Ok(Self {
            op: Operator::new(builder)?.finish(),
            write_handles: DashMap::with_capacity(2),
        })
    }

    #[inline(always)]
    pub fn records_path(
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
        offset: u64,
    ) -> String {
        format!(
            "records/{}/{}/record-{:020}",
            namespace.as_ref(),
            shard_name.as_ref(),
            offset
        )
    }

    #[inline(always)]
    pub fn offsets_path(namespace: impl AsRef<str>, shard_name: impl AsRef<str>) -> String {
        format!(
            "offsets/{}-{}-offset",
            namespace.as_ref(),
            shard_name.as_ref()
        )
    }

    #[inline(always)]
    pub fn shard_config_path(namespace: impl AsRef<str>, shard_name: impl AsRef<str>) -> String {
        format!(
            "shard_config/{}-{}-config",
            namespace.as_ref(),
            shard_name.as_ref()
        )
    }

    #[inline(always)]
    pub fn tags_path(
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
        tag: impl AsRef<str>,
        offset: u64,
    ) -> String {
        format!(
            "tags/{}/{}/{}/record-{:020}",
            namespace.as_ref(),
            shard_name.as_ref(),
            tag.as_ref(),
            offset
        )
    }

    #[inline(always)]
    pub fn tags_path_prefix(
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
        tag: impl AsRef<str>,
    ) -> String {
        format!(
            "tags/{}/{}/{}/",
            namespace.as_ref(),
            shard_name.as_ref(),
            tag.as_ref()
        )
    }

    #[inline(always)]
    pub fn key_path(
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> String {
        format!(
            "keys/{}/{}/key-{}",
            namespace.as_ref(),
            shard_name.as_ref(),
            key.as_ref()
        )
    }

    #[inline(always)]
    pub fn group_path(
        group_name: impl AsRef<str>,
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
    ) -> String {
        format!(
            "groups/{}/{}-{}",
            group_name.as_ref(),
            namespace.as_ref(),
            shard_name.as_ref()
        )
    }

    #[inline(always)]
    pub fn group_path_prefix(group_name: impl AsRef<str>) -> String {
        format!("groups/{}/", group_name.as_ref())
    }
}

impl MinIoStorageAdapter {
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

        timeout(Duration::from_secs(3600), resp_rx)
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

        Self::spawn_write_thread(self.op.clone(), stop_recv, data_recv).await;

        let write_handle = ThreadWriteHandle {
            data_sender,
            stop_sender,
        };

        self.register_write_handle(namespace.as_ref(), shard_name.as_ref(), write_handle)
            .await;
    }

    async fn spawn_write_thread(
        op: Operator,
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
                            thread_batch_write(op.clone(), packet.namespace, packet.shard, packet.records)
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
        op: Operator,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let mut offsets = Vec::new();

        let start_offset_bytes = op
            .read(&Self::offsets_path(&namespace, &shard_name))
            .await?
            .to_vec();

        let mut start_offset = serde_json::from_slice::<u64>(&start_offset_bytes)?;

        for mut message in messages {
            message.offset = Some(start_offset);
            // write records
            let record_path = Self::records_path(&namespace, &shard_name, start_offset);
            op.write(&record_path, serde_json::to_vec(&message)?)
                .await?;

            // write key
            let key_path = Self::key_path(&namespace, &shard_name, &message.key);
            op.write(&key_path, serde_json::to_vec(&message)?).await?;

            // write tags
            for tag in message.tags.iter() {
                let tag_path = Self::tags_path(&namespace, &shard_name, tag, start_offset);
                op.write(&tag_path, serde_json::to_vec(&message)?).await?;
            }

            offsets.push(start_offset);
            start_offset += 1;
        }

        op.write(
            &Self::offsets_path(&namespace, &shard_name),
            serde_json::to_vec(&start_offset)?,
        )
        .await?;

        Ok(offsets)
    }
}

#[async_trait]
impl StorageAdapter for MinIoStorageAdapter {
    async fn create_shard(
        &self,
        namespace: String,
        shard_name: String,
        shard_config: ShardConfig,
    ) -> Result<(), CommonError> {
        self.op
            .write(
                &Self::offsets_path(&namespace, &shard_name),
                serde_json::to_vec(&0)?,
            )
            .await?;

        self.op
            .write(
                &Self::shard_config_path(&namespace, &shard_name),
                serde_json::to_vec(&shard_config)?,
            )
            .await?;

        Ok(())
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        self.op
            .remove_all(&Self::offsets_path(&namespace, &shard_name))
            .await?;
        Ok(())
    }

    async fn write(
        &self,
        namespace: String,
        shard_name: String,
        data: Record,
    ) -> Result<u64, CommonError> {
        let offsets = self
            .handle_write_request(namespace, shard_name, vec![data])
            .await?;

        Ok(offsets[0])
    }

    async fn batch_write(
        &self,
        namespace: String,
        shard_name: String,
        data: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        self.handle_write_request(namespace, shard_name, data).await
    }

    async fn read_by_offset(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let mut res = Vec::new();
        let mut total_bytes = 0;

        for i in offset..offset + read_config.max_record_num {
            let path = Self::records_path(&namespace, &shard_name, i);
            if !self.op.exists(&path).await? {
                break;
            }
            let record_bytes = self.op.read(&path).await?.to_vec();
            if record_bytes.len() + total_bytes > read_config.max_size as usize {
                break;
            }
            total_bytes += record_bytes.len();
            let record = serde_json::from_slice::<Record>(&record_bytes)?;
            res.push(record);
        }

        Ok(res)
    }

    async fn read_by_tag(
        &self,
        namespace: String,
        shard_name: String,
        start_offset: u64,
        tag: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let tags_path_prefix = Self::tags_path_prefix(&namespace, &shard_name, &tag);

        let mut lister = self.op.lister_with(&tags_path_prefix).await?;

        let mut records = Vec::new();
        let mut bytes_read = 0;

        while let Some(entry) = lister.try_next().await? {
            match entry.metadata().mode() {
                EntryMode::FILE => {
                    if records.len() >= read_config.max_record_num as usize {
                        break;
                    }

                    let path = entry.path();
                    let record_bytes = self.op.read(path).await?.to_vec();

                    if bytes_read + record_bytes.len() > read_config.max_size as usize {
                        break;
                    }

                    let record = serde_json::from_slice::<Record>(&record_bytes)?;

                    if record.offset.unwrap() < start_offset {
                        continue;
                    }

                    records.push(record);
                    bytes_read += record_bytes.len();
                }
                _ => continue,
            }
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
        if read_config.max_record_num == 0 {
            return Ok(vec![]);
        }

        let record_bytes = self
            .op
            .read(&Self::key_path(&namespace, &shard_name, key))
            .await?
            .to_vec();

        if record_bytes.len() < read_config.max_size as usize {
            return Ok(vec![]);
        }

        let record = serde_json::from_slice::<Record>(&record_bytes)?;

        if record.offset.unwrap() < offset {
            return Ok(vec![]);
        }

        Ok(vec![record])
    }

    async fn get_offset_by_timestamp(
        &self,
        _namespace: String,
        _shard_name: String,
        _timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        Ok(None)
    }

    async fn get_offset_by_group(
        &self,
        group_name: String,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        if self
            .op
            .exists(&Self::group_path_prefix(&group_name))
            .await?
        {
            let mut offsets = Vec::new();
            let mut lister = self
                .op
                .lister_with(&Self::group_path_prefix(&group_name))
                .await?;

            while let Some(entry) = lister.try_next().await? {
                match entry.metadata().mode() {
                    EntryMode::FILE => {
                        let path = entry.path();

                        let offset_bytes = self.op.read(path).await?.to_vec();
                        let offset = serde_json::from_slice::<u64>(&offset_bytes)?;
                        offsets.push(ShardOffset {
                            offset,
                            ..Default::default()
                        });
                    }
                    _ => continue,
                }
            }

            Ok(offsets)
        } else {
            Ok(vec![])
        }
    }

    async fn commit_offset(
        &self,
        group_name: String,
        namespace: String,
        offset: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        for (shard_name, offset) in offset {
            self.op
                .write(
                    &Self::group_path(&group_name, &namespace, &shard_name),
                    serde_json::to_vec(&offset)?,
                )
                .await?;
        }

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

    use common_base::tools::unique_id;
    use futures::future;
    use metadata_struct::adapter::{
        read_config::ReadConfig,
        record::{Header, Record},
    };

    use crate::{
        minio::MinIoStorageAdapter,
        storage::{ShardConfig, StorageAdapter},
    };

    #[tokio::test]
    #[ignore]
    async fn stream_read_write() {
        let storage_adapter = MinIoStorageAdapter::new("/tmp/minio", "test").unwrap();
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
                    max_record_num: 10,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert_eq!(result_read.len(), 2);

        // test group functionalities
        let group_id = format!("group-{}", unique_id());
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
            .delete_shard(namespace, shard_name)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn concurrency_test() {
        let storage_adapter = Arc::new(MinIoStorageAdapter::new("/tmp/minio", "test").unwrap());

        // create one namespace with 10 shards
        let namespace = unique_id();
        let shards = (0..10).map(|i| format!("test-{}", i)).collect::<Vec<_>>();

        // create shards
        for i in 0..shards.len() {
            storage_adapter
                .create_shard(
                    namespace.clone(),
                    shards.get(i).unwrap().clone(),
                    ShardConfig::default(),
                )
                .await
                .unwrap();
        }

        let header = vec![Header {
            name: "name".to_string(),
            value: "value".to_string(),
        }];

        // create 100 tokio tasks, each of which will write 100 records to a shard
        let mut tasks = vec![];
        for tid in 0..100 {
            let storage_adapter = storage_adapter.clone();
            let namespace = namespace.clone();
            let shard_name = shards.get(tid % shards.len()).unwrap().clone();
            let header = header.clone();

            let task = tokio::spawn(async move {
                let mut batch_data = Vec::new();

                for idx in 0..100 {
                    let data = Record {
                        offset: None,
                        header: header.clone(),
                        key: format!("key-{}-{}", tid, idx),
                        data: format!("data-{}-{}", tid, idx).as_bytes().to_vec(),
                        tags: vec![format!("task-{}", tid)],
                        timestamp: 0,
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

            assert_eq!(len, (100 / shards.len()) * 100);
        }
    }
}
