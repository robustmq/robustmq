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

use std::collections::HashMap;

use axum::async_trait;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;

use crate::storage::{ShardConfig, ShardOffset, StorageAdapter};

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    pub shard_data: DashMap<String, Vec<Record>>,
    //group, (namespace_shard_name,offset)
    pub group_data: DashMap<String, DashMap<String, u64>>,
}

impl Default for MemoryStorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorageAdapter {
    pub fn new() -> Self {
        MemoryStorageAdapter {
            shard_data: DashMap::with_capacity(256),
            group_data: DashMap::with_capacity(256),
        }
    }

    pub fn shard_key(&self, namespace: &str, shard_name: &str) -> String {
        format!("{}_{}", namespace, shard_name)
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
            .insert(self.shard_key(&namespace, &shard_name), Vec::new());
        return Ok(());
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        self.shard_data
            .remove(&self.shard_key(&namespace, &shard_name));
        return Ok(());
    }

    async fn batch_write(
        &self,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let shard_key = self.shard_key(&namespace, &shard_name);
        let mut offset_res = Vec::new();

        if let Some(mut data_list) = self.shard_data.get_mut(&shard_key) {
            let mut start_offset = data_list.len();
            for mut msg in messages {
                offset_res.push(start_offset as u64);
                msg.offset = Some(start_offset as u64);
                data_list.push(msg);
                start_offset += 1;
            }
        } else {
            let mut data_list = Vec::new();
            for (offset, mut msg) in messages.into_iter().enumerate() {
                offset_res.push(offset as u64);

                msg.offset = Some(offset as u64);
                data_list.push(msg);
            }
            self.shard_data.insert(shard_key, data_list);
        }

        return Ok(offset_res);
    }

    async fn write(
        &self,
        namespace: String,
        shard_name: String,
        mut data: Record,
    ) -> Result<u64, CommonError> {
        let shard_key = self.shard_key(&namespace, &shard_name);

        let offset = if let Some(mut data_list) = self.shard_data.get_mut(&shard_key) {
            let start_offset = data_list.len();

            data.offset = Some(start_offset as u64);
            data_list.push(data);

            start_offset
        } else {
            data.offset = Some(0);
            self.shard_data.insert(shard_key, vec![data]);
            0
        };

        return Ok(offset as u64);
    }

    async fn read_by_offset(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_key = self.shard_key(&namespace, &shard_name);

        if let Some(data_list) = self.shard_data.get(&shard_key) {
            if data_list.len() < offset as usize {
                return Ok(Vec::new());
            }

            let mut result = Vec::new();
            for i in offset..(offset + read_config.max_record_num) {
                if let Some(value) = data_list.get(i as usize) {
                    result.push(value.clone());
                } else {
                    break;
                }
            }
            return Ok(result);
        }

        Ok(Vec::new())
    }

    async fn read_by_tag(
        &self,
        _namespace: String,
        _shard_name: String,
        _offset: u64,
        _tag: String,
        _read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        return Ok(Vec::new());
    }

    async fn read_by_key(
        &self,
        _namespace: String,
        _shard_name: String,
        _key: String,
        _read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        return Ok(Vec::new());
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
        let mut results = Vec::new();
        if let Some(data) = self.group_data.get(&group_name) {
            for raw in data.iter() {
                results.push(ShardOffset {
                    offset: *raw.value(),
                    ..Default::default()
                });
            }
        }

        Ok(results)
    }

    async fn commit_offset(
        &self,
        group_name: String,
        namespace: String,
        offset: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        if let Some(data) = self.group_data.get_mut(&group_name) {
            for (shard_name, offset) in offset.iter() {
                let group_key = self.shard_key(&namespace, shard_name);
                data.insert(group_key, *offset);
            }
        } else {
            let data = DashMap::with_capacity(2);
            for (shard_name, offset) in offset.iter() {
                let group_key = self.shard_key(&namespace, shard_name);
                data.insert(group_key, *offset);
            }
            self.group_data.insert(group_name, data);
        }
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use common_base::tools::unique_id;
    use metadata_struct::adapter::read_config::ReadConfig;
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
            Record::build_byte(ms1.clone().as_bytes().to_vec()),
            Record::build_byte(ms2.clone().as_bytes().to_vec()),
        ];
        let namespace = unique_id();
        let shard_key = storage_adapter.shard_key(&namespace, &shard_name);

        let result = storage_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();
        assert_eq!(result.first().unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);
        assert!(storage_adapter.shard_data.contains_key(&shard_key));
        assert_eq!(storage_adapter.shard_data.get(&shard_key).unwrap().len(), 2);

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
        assert_eq!(result.first().unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert!(storage_adapter.shard_data.contains_key(&shard_key));
        assert_eq!(storage_adapter.shard_data.get(&shard_key).unwrap().len(), 4);

        let group_id = "test_group_id".to_string();
        let mut read_config = ReadConfig::new();
        read_config.max_record_num = 1;

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

        // read m2
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
        offset_data.insert(shard_name, res.first().unwrap().clone().offset.unwrap());
        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();
    }
}
