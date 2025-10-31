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

use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    pub shard_info: DashMap<String, ShardInfo>,
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
            shard_info: DashMap::with_capacity(2),
        }
    }

    pub fn shard_key(&self, namespace: &str, shard_name: &str) -> String {
        format!("{namespace}_{shard_name}")
    }
}

impl MemoryStorageAdapter {}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        let key = self.shard_key(&shard.namespace, &shard.shard_name);
        self.shard_data.insert(key.clone(), Vec::new());
        self.shard_info.insert(key.clone(), shard.clone());
        Ok(())
    }

    async fn list_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<ShardInfo>, CommonError> {
        if !shard_name.is_empty() {
            let key = self.shard_key(namespace, shard_name);
            if let Some(info) = self.shard_info.get(&key) {
                return Ok(vec![info.clone()]);
            }
        }

        Ok(self.shard_info.iter().map(|v| v.value().clone()).collect())
    }

    async fn delete_shard(&self, namespace: &str, shard_name: &str) -> Result<(), CommonError> {
        self.shard_data
            .remove(&self.shard_key(namespace, shard_name));
        Ok(())
    }

    async fn batch_write(
        &self,
        namespace: &str,
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);
        let mut offset_res = Vec::new();

        if let Some(mut data_list) = self.shard_data.get_mut(&shard_key) {
            let mut start_offset = data_list.len();
            for msg in messages {
                offset_res.push(start_offset as u64);
                let mut msg_clone = msg.clone();
                msg_clone.offset = Some(start_offset as u64);
                data_list.push(msg_clone);
                start_offset += 1;
            }
        } else {
            let mut data_list = Vec::new();
            for (offset, msg) in messages.iter().enumerate() {
                offset_res.push(offset as u64);
                let mut msg_clone = msg.clone();
                msg_clone.offset = Some(offset as u64);
                data_list.push(msg_clone);
            }
            self.shard_data.insert(shard_key, data_list);
        }

        Ok(offset_res)
    }

    async fn write(
        &self,
        namespace: &str,
        shard_name: &str,
        data: &Record,
    ) -> Result<u64, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

        let offset = if let Some(mut data_list) = self.shard_data.get_mut(&shard_key) {
            let start_offset = data_list.len();
            let mut data_clone = data.clone();
            data_clone.offset = Some(start_offset as u64);
            data_list.push(data_clone);

            start_offset
        } else {
            let mut data_clone = data.clone();
            data_clone.offset = Some(0);
            self.shard_data.insert(shard_key, vec![data_clone]);
            0
        };

        Ok(offset as u64)
    }

    async fn read_by_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

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
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    async fn read_by_tag(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

        if let Some(record_list) = self.shard_data.get(&shard_key) {
            if record_list.len() < offset as usize {
                return Ok(Vec::new());
            }
            let mut result = Vec::new();

            for i in offset..(offset + read_config.max_record_num) {
                if let Some(value) = record_list.get(i as usize) {
                    if value.tags.contains(&tag.to_string()) {
                        result.push(value.clone());
                    }
                } else {
                    break;
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    async fn read_by_key(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

        if let Some(record_list) = self.shard_data.get(&shard_key) {
            if record_list.len() < offset as usize {
                return Ok(Vec::new());
            }
            let mut result = Vec::new();

            for i in offset..(offset + read_config.max_record_num) {
                if let Some(value) = record_list.get(i as usize) {
                    if value.key == key {
                        result.push(value.clone());
                    }
                } else {
                    break;
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_offset_by_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

        if let Some(record_list) = self.shard_data.get(&shard_key) {
            for record in record_list.iter() {
                if record.timestamp >= timestamp {
                    if record.offset.is_none() {
                        return Ok(None);
                    }
                    return Ok(Some(ShardOffset {
                        offset: record.offset.unwrap(),
                        ..Default::default()
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
        let mut results = Vec::new();
        if let Some(data) = self.group_data.get(group_name) {
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
        group_name: &str,
        namespace: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        if let Some(data) = self.group_data.get_mut(group_name) {
            for (shard_name, offset) in offset.iter() {
                let group_key = self.shard_key(namespace, shard_name);
                data.insert(group_key, *offset);
            }
        } else {
            let data = DashMap::with_capacity(2);
            for (shard_name, offset) in offset.iter() {
                let group_key = self.shard_key(namespace, shard_name);
                data.insert(group_key, *offset);
            }
            self.group_data.insert(group_name.to_string(), data);
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
            .batch_write(&namespace, &shard_name, &data)
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
            .batch_write(&namespace, &shard_name, &data)
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
            .read_by_offset(&namespace, &shard_name, offset, &read_config)
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
            .commit_offset(&group_id, &namespace, &offset_data)
            .await
            .unwrap();

        // read m2
        let offset = storage_adapter
            .get_offset_by_group(&group_id)
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                &namespace,
                &shard_name,
                offset.first().unwrap().offset + 1,
                &read_config,
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
            .commit_offset(&group_id, &namespace, &offset_data)
            .await
            .unwrap();

        // read m3
        let offset: Vec<crate::storage::ShardOffset> = storage_adapter
            .get_offset_by_group(&group_id)
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                &namespace,
                &shard_name,
                offset.first().unwrap().offset + 1,
                &read_config,
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
            .commit_offset(&group_id, &namespace, &offset_data)
            .await
            .unwrap();

        // read m4
        let offset = storage_adapter
            .get_offset_by_group(&group_id)
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                &namespace,
                &shard_name,
                offset.first().unwrap().offset + 1,
                &read_config,
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
            .commit_offset(&group_id, &namespace, &offset_data)
            .await
            .unwrap();
    }
}
