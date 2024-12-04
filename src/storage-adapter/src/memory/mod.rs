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

use axum::async_trait;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::record::Record;

use crate::storage::{ShardConfig, StorageAdapter};

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    pub memory_data: DashMap<String, Record>,
    pub shard_data: DashMap<String, Vec<Record>>,
    pub group_data: DashMap<String, u128>,
    pub key_index: DashMap<String, DashMap<String, u128>>,
}

impl Default for MemoryStorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorageAdapter {
    pub fn new() -> Self {
        MemoryStorageAdapter {
            memory_data: DashMap::with_capacity(256),
            shard_data: DashMap::with_capacity(256),
            group_data: DashMap::with_capacity(256),
            key_index: DashMap::with_capacity(256),
        }
    }

    pub fn shard_key(&self, namespace: String, shard_name: String) -> String {
        format!("{}_{}", namespace, shard_name)
    }

    pub fn offset_key(&self, namespace: String, group_id: String, shard_name: String) -> String {
        format!("{}_{}_{}", namespace, group_id, shard_name)
    }

    pub fn get_offset(
        &self,
        namespace: String,
        group_id: String,
        shard_name: String,
    ) -> Option<u128> {
        let key = self.offset_key(namespace, group_id, shard_name);
        if let Some(offset) = self.group_data.get(&key) {
            return Some(*offset);
        }
        None
    }
}

impl MemoryStorageAdapter {}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn create_shard(
        &self,
        namespace: String,
        shard_name: String,
        _: ShardConfig,
    ) -> Result<(), CommonError> {
        self.shard_data
            .insert(self.shard_key(namespace, shard_name), Vec::new());
        return Ok(());
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        self.shard_data
            .remove(&self.shard_key(namespace, shard_name));
        return Ok(());
    }

    async fn stream_write(
        &self,
        namespace: String,
        shard_name: String,
        message: Vec<Record>,
    ) -> Result<Vec<usize>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);
        let mut shard = if let Some((_, da)) = self.shard_data.remove(&shard_key) {
            da
        } else {
            Vec::new()
        };
        let mut start_offset = shard.len();
        let mut record_list = Vec::new();
        let mut offset_res = Vec::new();
        for mut msg in message {
            offset_res.push(start_offset);
            msg.offset = start_offset as u128;
            start_offset += 1;
            record_list.push(msg);
            // todo build timestamp index
            // todo build key index
        }

        shard.append(&mut record_list);
        self.shard_data.insert(shard_key, shard);
        return Ok(offset_res);
    }

    async fn stream_read(
        &self,
        namespace: String,
        shard_name: String,
        group_id: String,
        record_num: Option<u128>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError> {
        let offset = if let Some(da) = self.get_offset(namespace, group_id, shard_name.clone()) {
            da + 1
        } else {
            0
        };

        let num = record_num.unwrap_or(10);
        if let Some(da) = self.shard_data.get(&shard_name) {
            let mut cur_offset = 0;
            let mut result = Vec::new();
            for i in offset..(offset + num) {
                if let Some(value) = da.get(i as usize) {
                    result.push(value.clone());
                    cur_offset += 1;
                } else {
                    break;
                }
            }
            if cur_offset > 0 {
                self.group_data.insert(shard_name, offset + cur_offset);
            }
            return Ok(Some(result));
        }
        return Ok(None);
    }

    async fn stream_commit_offset(
        &self,
        namespace: String,
        shard_name: String,
        group_id: String,
        offset: u128,
    ) -> Result<bool, CommonError> {
        let key = self.offset_key(namespace, group_id, shard_name);
        self.group_data.insert(key, offset);
        return Ok(true);
    }

    async fn stream_read_by_offset(
        &self,
        namespace: String,
        shard_name: String,
        offset: usize,
    ) -> Result<Option<Record>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);
        if let Some(da) = self.shard_data.get(&shard_key) {
            if let Some(value) = da.get(offset) {
                return Ok(Some(value.clone()));
            }
        }
        return Ok(None);
    }

    async fn stream_read_by_timestamp(
        &self,
        _: String,
        _: String,
        _: u128,
        _: u128,
        _: Option<usize>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError> {
        return Ok(None);
    }

    async fn stream_read_by_key(
        &self,
        _: String,
        _: String,
        _: String,
    ) -> Result<Option<Record>, CommonError> {
        return Ok(None);
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common_base::tools::unique_id;
    use metadata_struct::adapter::record::Record;

    use super::MemoryStorageAdapter;
    use crate::storage::StorageAdapter;

    #[tokio::test]
    async fn stream_read_write() {
        let storage_adapter = MemoryStorageAdapter::new();
        let shard_name = "test-11".to_string();
        let ms1 = "test1".to_string();
        let ms2 = "test2".to_string();
        let data = vec![
            Record::build_b(ms1.clone().as_bytes().to_vec()),
            Record::build_b(ms2.clone().as_bytes().to_vec()),
        ];
        let namespace = unique_id();

        let result = storage_adapter
            .stream_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();
        assert_eq!(result.first().unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);
        assert!(storage_adapter.shard_data.contains_key(&shard_name));
        assert_eq!(
            storage_adapter.shard_data.get(&shard_name).unwrap().len(),
            2
        );

        let ms3 = "test3".to_string();
        let ms4 = "test4".to_string();
        let data = vec![
            Record::build_b(ms3.clone().as_bytes().to_vec()),
            Record::build_b(ms4.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .stream_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();
        assert_eq!(result.first().unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert!(storage_adapter.shard_data.contains_key(&shard_name));
        assert_eq!(
            storage_adapter.shard_data.get(&shard_name).unwrap().len(),
            4
        );

        let group_id = "test_group_id".to_string();
        let record_num = Some(1);
        let record_size = None;
        let res = storage_adapter
            .stream_read(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms1
        );
        storage_adapter
            .stream_commit_offset(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                res.first().unwrap().clone().offset,
            )
            .await
            .unwrap();

        let res = storage_adapter
            .stream_read(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms2
        );
        storage_adapter
            .stream_commit_offset(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                res.first().unwrap().clone().offset,
            )
            .await
            .unwrap();

        let res = storage_adapter
            .stream_read(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms3
        );
        storage_adapter
            .stream_commit_offset(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                res.first().unwrap().clone().offset,
            )
            .await
            .unwrap();

        let res = storage_adapter
            .stream_read(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms4
        );
        storage_adapter
            .stream_commit_offset(
                namespace.clone(),
                shard_name.clone(),
                group_id.clone(),
                res.first().unwrap().clone().offset,
            )
            .await
            .unwrap();
    }
}
