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

use crate::expire::MessageExpireConfig;
use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::{error::common::CommonError, utils::crc::calc_crc32};
use common_config::storage::mysql::StorageDriverMySQLConfig;
use metadata_struct::adapter::record::{Header as AdapterHeader, StorageAdapterRecord};
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::storage::convert::convert_adapter_headers_to_storage;
use metadata_struct::storage::record::{Header as StorageHeader, StorageEngineRecord, StorageEngineRecordMetadata};
use r2d2_mysql::mysql::{params, prelude::Queryable, Row};
use std::{collections::HashMap, time::Duration};
use third_driver::mysql::{build_mysql_conn_pool, MysqlPool};
use tokio::{
    sync::mpsc::{self, Receiver},
    time::sleep,
};

#[derive(Clone)]
pub struct MySQLStorageAdapter {
    pool: MysqlPool,
    stop_send: mpsc::Sender<bool>,
}

impl MySQLStorageAdapter {
    pub fn new(config: StorageDriverMySQLConfig) -> Result<Self, CommonError> {
        let pool = build_mysql_conn_pool(&config.mysql_addr)?;
        // init tags and groups table
        let mut conn = pool.get()?;

        let create_tags_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `shard` varchar(255) NOT NULL,
                `m_offset` bigint unsigned NOT NULL,
                `tag` varchar(255) NOT NULL,
                PRIMARY KEY (`m_offset`, `tag`),
                INDEX `ns_shard_tag_offset_idx` (`shard`, `tag`, `m_offset`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::tags_table_name()
        );

        conn.query_drop(create_tags_table_sql)?;

        let create_groups_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `group` varchar(255) NOT NULL,
                `shard` varchar(255) NOT NULL,
                `offset` bigint unsigned NOT NULL,
                PRIMARY KEY (`group`,  `shard`),
                INDEX `group` (`group`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::groups_table_name()
        );

        conn.query_drop(create_groups_table_sql)?;

        let create_shard_info_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `shard` varchar(255) NOT NULL,
                `info` blob,
                PRIMARY KEY (`shard`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::shard_info_table_name()
        );

        conn.query_drop(create_shard_info_table_sql)?;

        let (stop_send, stop_recv) = mpsc::channel(1);

        Self::spawn_clean_thread(pool.clone(), stop_recv);

        Ok(MySQLStorageAdapter { pool, stop_send })
    }

    #[inline(always)]
    pub fn record_table_name(shard_name: impl AsRef<str>) -> String {
        format!("record_{}", shard_name.as_ref())
    }

    #[inline(always)]
    pub fn tags_table_name() -> String {
        "tags".to_string()
    }

    #[inline(always)]
    pub fn groups_table_name() -> String {
        "groups".to_string()
    }

    #[inline(always)]
    pub fn shard_info_table_name() -> String {
        "shard_info".to_string()
    }

    #[inline(always)]
    pub fn increment_id_table_name(shard_name: impl AsRef<str>) -> String {
        format!("increment_id_{}", shard_name.as_ref())
    }
}

impl MySQLStorageAdapter {
    async fn handle_write_request(
        &self,
        shard_name: &str,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let mut conn = self.pool.get()?;

        let mut offsets = Vec::new();

        for message in messages {
            // STEP 1: insert an empty record in the increment_id table to get an offset
            conn.query_drop(format!(
                "INSERT INTO `{}` () VALUES ();",
                Self::increment_id_table_name(shard_name)
            ))?;

            let offset: u64 =
                conn.query_first("SELECT LAST_INSERT_ID();")?
                    .ok_or(CommonError::CommonError(
                        "Failed to get last insert ID".to_string(),
                    ))?;

            let insert_record_sql = format!(
                "INSERT INTO `{}` (`offset`, `key`, `data`, `header`, `tags`, `ts`) VALUES (:offset, :key, :data, :header, :tags, :ts);",
                Self::record_table_name(shard_name)
            );

            conn.exec_drop(
                insert_record_sql,
                params! {
                    "offset" => offset - 1,   // offset is 1-based in the mysql AUTO_INCREMENT column
                    "key" => message.key.clone().unwrap_or_default(),
                    "data" => message.data.to_vec(),
                    "header" => serde_json::to_vec(&message.header)?,
                    "tags" => serde_json::to_vec(&message.tags)?,
                    "ts" => message.timestamp,
                },
            )?;

            offsets.push(offset - 1);

            // Then we need to insert into tags table
            let insert_tags_sql = format!(
                "INSERT INTO `{}` (`shard`, `m_offset`, `tag`) VALUES (:shard, :m_offset, :tag);",
                Self::tags_table_name()
            );

            if let Some(tags) = &message.tags {
                conn.exec_batch(
                    insert_tags_sql.as_str(),
                    tags.iter().map(|tag| {
                        params! {
                            "shard" => shard_name.to_string(),
                            "m_offset" => offset - 1, // offset is 1-based in the mysql AUTO_INCREMENT column
                            "tag" => tag,
                        }
                    }),
                )?;
            }
        }

        Ok(offsets)
    }

    fn spawn_clean_thread(pool: MysqlPool, mut stop_recv: Receiver<bool>) {
        tokio::spawn(async move {
            loop {
                if let Ok(true) = stop_recv.try_recv() {
                    break;
                }

                let mut conn = pool.get().unwrap();

                let sql = format!("SELECT info FROM `{}`;", Self::shard_info_table_name());
                let query_res = conn
                    .query_map(sql, |info: Vec<u8>| {
                        serde_json::from_slice::<ShardInfo>(&info).unwrap()
                    })
                    .unwrap();

                // Clean up the increment_id table every 10 minutes
                for shard_info in query_res {
                    let incr_id_table_name =
                        Self::increment_id_table_name(shard_info.shard_name.clone());
                    conn.query_drop(format!(
                        "DELETE FROM `{}` WHERE `offset` < (
                            SELECT max_offset FROM (
                                SELECT MAX(offset) - 10 AS max_offset FROM `{}`
                            ) AS subquery
                        );",
                        incr_id_table_name.clone(),
                        incr_id_table_name
                    ))
                    .unwrap();
                }

                sleep(Duration::from_secs(600)).await;
            }
        });
    }
}

#[async_trait]
impl StorageAdapter for MySQLStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        let mut conn = self.pool.get()?;

        let table_name = Self::record_table_name(&shard.shard_name);

        let check_table_exists_sql = format!("SHOW TABLES LIKE '{table_name}';");

        if conn
            .query_first::<Row, _>(check_table_exists_sql)?
            .is_some()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} already exists",
                &shard.shard_name
            )));
        };

        let create_table_sql = format!(
            "CREATE TABLE `{table_name}` (
                `offset` bigint unsigned PRIMARY KEY,
                `key` varchar(255) DEFAULT NULL,
                `data` blob,
                `header` blob,
                `tags` blob,
                `ts` bigint unsigned NOT NULL,
                INDEX `key_idx` (`key`),
                INDEX `ts_idx` (`ts`),
                INDEX `ts_offset_idx` (`ts`, `offset`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;"
        );

        conn.query_drop(create_table_sql)?;

        let insert_shard_info_sql = format!(
            "REPLACE INTO `{}` (`shard`, `info`) VALUES ( :shard, :info)",
            Self::shard_info_table_name()
        );

        conn.exec_drop(
            insert_shard_info_sql,
            params! {
                "shard" => shard.shard_name.clone(),
                "info" => serde_json::to_vec(&shard)?,
            },
        )?;

        // create a dummy sql table with only one auto increment column
        let create_increment_id_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `offset` bigint unsigned PRIMARY KEY AUTO_INCREMENT
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::increment_id_table_name(&shard.shard_name)
        );

        conn.query_drop(create_increment_id_table_sql)?;

        Ok(())
    }

    async fn list_shard(&self, shard: &str) -> Result<Vec<ShardInfo>, CommonError> {
        let mut conn = self.pool.get()?;

        if shard.is_empty() {
            let sql = format!("SELECT info FROM `{}`", Self::shard_info_table_name());

            let res = conn.query_map(sql, |info: Vec<u8>| {
                serde_json::from_slice::<ShardInfo>(&info).unwrap()
            })?;

            return Ok(res);
        }

        let sql = format!(
            "SELECT info FROM `{}` WHERE shard = :shard;",
            Self::shard_info_table_name()
        );

        let res = conn.exec_map(
            sql,
            params! {
                "shard" => shard,
            },
            |info: Vec<u8>| serde_json::from_slice::<ShardInfo>(&info).unwrap(),
        )?;

        Ok(res)
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        let table_name = Self::record_table_name(shard);

        let check_table_exists_sql = format!("SHOW TABLES LIKE '{table_name}';");

        let mut conn = self.pool.get()?;

        if conn
            .query_first::<Row, _>(check_table_exists_sql)?
            .is_none()
        {
            return Err(CommonError::CommonError(format!(
                "shard {}  does not exist",
                shard
            )));
        };

        let drop_table_sql = format!("DROP TABLE IF EXISTS `{table_name}`");

        conn.query_drop(drop_table_sql)?;

        let drop_shard_info_sql = format!(
            "DELETE FROM `{}` WHERE shard = :shard",
            Self::shard_info_table_name()
        );

        conn.exec_drop(
            drop_shard_info_sql,
            params! {
                "shard" => shard,
            },
        )?;

        Ok(())
    }

    async fn write(&self, shard: &str, data: &Record) -> Result<u64, CommonError> {
        let offsets = self.handle_write_request(shard, vec![data.clone()]).await?;

        Ok(offsets.first().cloned().ok_or(CommonError::CommonError(
            "Failed to get offset. The vector is empty!".to_string(),
        ))?)
    }

    async fn batch_write(&self, shard: &str, data: &[Record]) -> Result<Vec<u64>, CommonError> {
        self.handle_write_request(shard, data.to_vec()).await
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<StorageEngineRecord>, CommonError> {
        let mut conn = self.pool.get()?;

        let sql = format!(
            "SELECT `offset`, `key`, `data`, `header`, `tags`, `ts`
            FROM `{}`
            WHERE `offset` >= :offset
            ORDER BY `offset`
            LIMIT :limit;",
            Self::record_table_name(shard)
        );

        let res: Vec<StorageEngineRecord> = conn.exec_map(
            sql,
            params! {
                "offset" => offset,
                "limit" => read_config.max_record_num,
            },
            |(offset, key, data, header, tags, ts): (
                u64,
                String,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                u64,
            )| {
                let data_bytes = data.into();
                let adapter_headers: Option<Vec<AdapterHeader>> = serde_json::from_slice(&header).ok();
                let storage_headers = convert_adapter_headers_to_storage(adapter_headers);
                let tags_val = serde_json::from_slice(&tags).ok();
                let key_val = if key.is_empty() { None } else { Some(key) };
                
                let metadata = StorageEngineRecordMetadata::build(offset, shard.to_string(), 0)
                    .with_header(storage_headers)
                    .with_key(key_val)
                    .with_tags(tags_val)
                    .with_timestamp(ts)
                    .with_crc_from_data(&data_bytes);

                StorageEngineRecord {
                    metadata,
                    data: data_bytes,
                }
            },
        )?;

        Ok(res)
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &ReadConfig,
    ) -> Result<Vec<StorageEngineRecord>, CommonError> {
        let mut conn = self.pool.get()?;
        let offset = start_offset.unwrap_or(0);

        let sql = format!(
            "SELECT `r.offset`,`r.key`,`r.data`,`r.header`,`r.tags`,`r.ts`
            FROM
                `{}` l LEFT JOIN `{}` r on l.m_offset = r.offset
            WHERE l.tag = :tag and l.m_offset >= :offset and l.shard = :shard
            ORDER BY l.m_offset
            LIMIT :limit",
            Self::tags_table_name(),
            Self::record_table_name(shard)
        );

        let res: Vec<StorageEngineRecord> = conn.exec_map(
            sql,
            params! {
                "tag" => tag,
                "offset" => offset,
                "shard" => shard,
                "limit" => read_config.max_record_num,
            },
            |(offset, key, data, header, tags, ts): (
                u64,
                String,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                u64,
            )| {
                let data_bytes = data.into();
                let adapter_headers: Option<Vec<AdapterHeader>> = serde_json::from_slice(&header).ok();
                let storage_headers = convert_adapter_headers_to_storage(adapter_headers);
                let tags_val = serde_json::from_slice(&tags).ok();
                let key_val = if key.is_empty() { None } else { Some(key) };
                
                let metadata = StorageEngineRecordMetadata::build(offset, shard.to_string(), 0)
                    .with_header(storage_headers)
                    .with_key(key_val)
                    .with_tags(tags_val)
                    .with_timestamp(ts)
                    .with_crc_from_data(&data_bytes);

                StorageEngineRecord {
                    metadata,
                    data: data_bytes,
                }
            },
        )?;

        Ok(res)
    }

    async fn read_by_key(
        &self,
        shard: &str,
        key: &str,
    ) -> Result<Vec<StorageEngineRecord>, CommonError> {
        let mut conn = self.pool.get()?;

        let sql = format!(
            "SELECT `offset`, `key`, `data`, `header`, `tags`, `ts`
            FROM {}
            WHERE `key` = :key
            ORDER BY `offset`
            LIMIT 1",
            Self::record_table_name(shard)
        );

        let res = conn
            .exec_first(
                sql,
                params! {
                    "key" => key,
                },
            )?
            .map(
                |(offset, key, data, header, tags, ts): (
                    u64,
                    String,
                    Vec<u8>,
                    Vec<u8>,
                    Vec<u8>,
                    u64,
                )| {
                    let data_bytes = data.into();
                    let adapter_headers: Option<Vec<AdapterHeader>> = serde_json::from_slice(&header).ok();
                    let storage_headers = adapter_headers.map(|hs| {
                        hs.into_iter()
                            .map(|h| StorageHeader {
                                name: h.name,
                                value: h.value,
                            })
                            .collect()
                    });
                    let tags_val = serde_json::from_slice(&tags).ok();
                    let key_val = if key.is_empty() { None } else { Some(key) };
                    
                    let metadata = StorageEngineRecordMetadata::build(offset, shard.to_string(), 0)
                        .with_header(storage_headers)
                        .with_key(key_val)
                        .with_tags(tags_val)
                        .with_timestamp(ts)
                        .with_crc_from_data(&data_bytes);

                    StorageEngineRecord {
                        metadata,
                        data: data_bytes,
                    }
                },
            )
            .ok_or(CommonError::CommonError("No record found".to_string()))?;

        Ok(vec![res])
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let mut conn = self.pool.get()?;

        let sql = format!(
            "SELECT `offset`
            FROM `{}`
            WHERE `ts` >= :ts
            ORDER BY `ts`
            LIMIT 1",
            Self::record_table_name(shard)
        );

        conn.exec_first(
            sql,
            params! {
                "ts" => timestamp,
            },
        )
        .map(|offset| {
            offset.map(|offset: u64| ShardOffset {
                offset,
                ..Default::default()
            })
        })
        .map_err(|e| CommonError::CommonError(format!("Failed to get offset by timestamp: {e}")))
    }

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
        let mut conn = self.pool.get()?;

        let sql = format!(
            "SELECT `offset`
            FROM `{}`
            WHERE `group` = :group",
            Self::groups_table_name()
        );

        let res = conn.exec_map(
            sql,
            params! {
                "group" => group_name,
            },
            |offset: u64| ShardOffset {
                offset,
                ..Default::default()
            },
        )?;

        Ok(res)
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let mut conn = self.pool.get()?;

        let sql = format!(
            "REPLACE INTO `{}` (`group`, `shard`, `offset`) VALUES (:group, :shard, :offset)",
            Self::groups_table_name()
        );

        conn.exec_batch(
            sql,
            offset.iter().map(|(shard_name, offset_val)| {
                params! {
                    "group" => group_name,
                    "shard" => shard_name,
                    "offset" => offset_val,
                }
            }),
        )?;

        Ok(())
    }

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        self.stop_send.send(true).await.map_err(|err| {
            CommonError::CommonError(format!("Failed to send stop signal: {err}"))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use super::MySQLStorageAdapter;
    use crate::storage::{ShardInfo, StorageAdapter};
    use common_base::{tools::unique_id, utils::crc::calc_crc32};
    use common_config::storage::mysql::StorageDriverMySQLConfig;
    use futures::future;
    use metadata_struct::adapter::{
        read_config::ReadConfig,
        record::{Header, Record},
    };

    #[tokio::test]
    #[ignore]
    async fn mysql_create_shard() {
        let mysql_adapter = MySQLStorageAdapter::new(StorageDriverMySQLConfig::default()).unwrap();
        let shard_name = String::from("test");
        let shard = ShardInfo {
            shard_name: shard_name.clone(),
            replica_num: 1,
            ..Default::default()
        };
        mysql_adapter.create_shard(&shard).await.unwrap();

        mysql_adapter.delete_shard(&shard.shard_name).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_batch_write() {
        let mysql_adapter = MySQLStorageAdapter::new(StorageDriverMySQLConfig::default()).unwrap();
        let shard_name = String::from("test");
        let shard = ShardInfo {
            shard_name: shard_name.clone(),
            replica_num: 1,
            ..Default::default()
        };
        mysql_adapter.create_shard(&shard).await.unwrap();
        let mut data = Vec::new();
        let header = vec![Header {
            name: "n1".to_string(),
            value: "v1".to_string(),
        }];

        let value = "test1".to_string().as_bytes().to_vec();
        data.push(Record {
            data: value.clone().into(),
            key: Some("k1".to_string()),
            header: Some(header.clone()),
            offset: None,
            timestamp: 1737600096,
            tags: None,
            crc_num: calc_crc32(&value),
        });

        let value = "test2".to_string().as_bytes().to_vec();
        data.push(Record {
            data: value.clone().into(),
            key: Some("k2".to_string()),
            header: Some(header.clone()),
            offset: None,
            timestamp: 1737600097,
            tags: None,
            crc_num: calc_crc32(&value),
        });

        let result = mysql_adapter
            .batch_write(&shard.shard_name, &data)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 1);

        let records = mysql_adapter
            .read_by_offset(
                &shard.shard_name,
                0,
                &ReadConfig {
                    max_record_num: 10,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].data, "test1".to_string().as_bytes().to_vec());
        assert_eq!(records[1].data, "test2".to_string().as_bytes().to_vec());

        assert_eq!(records[0].key, Some("k1".to_string()));
        assert_eq!(records[1].key, Some("k2".to_string()));

        assert_eq!(records[0].offset, Some(0));
        assert_eq!(records[1].offset, Some(1));
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_stream_read_write() {
        let mysql_adapter = MySQLStorageAdapter::new(StorageDriverMySQLConfig::default()).unwrap();
        let shard_name = "test-11".to_string();

        // step 1: create shard
        let shard = ShardInfo {
            shard_name: shard_name.clone(),
            replica_num: 1,
            ..Default::default()
        };
        mysql_adapter.create_shard(&shard).await.unwrap();

        let shards = mysql_adapter.list_shard(&shard.shard_name).await.unwrap();

        assert_eq!(shards.len(), 1);
        assert_eq!(shards.first().unwrap().shard_name, shard_name);
        assert_eq!(shards.first().unwrap().replica_num, 1);

        // insert two records (no key or tag) into the shard
        let ms1 = "test1".to_string();
        let ms2 = "test2".to_string();
        let data = vec![
            Record::from_bytes(ms1.clone().as_bytes().to_vec()),
            Record::from_bytes(ms2.clone().as_bytes().to_vec()),
        ];

        let result = mysql_adapter
            .batch_write(&shard.shard_name, &data)
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);

        // read previous records
        assert_eq!(
            mysql_adapter
                .read_by_offset(
                    &shard.shard_name,
                    0,
                    &ReadConfig {
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
            Record::from_bytes(ms3.clone().as_bytes().to_vec()),
            Record::from_bytes(ms4.clone().as_bytes().to_vec()),
        ];

        let result = mysql_adapter
            .batch_write(&shard.shard_name, &data)
            .await
            .unwrap();

        // read from offset 2
        let result_read = mysql_adapter
            .read_by_offset(
                &shard.shard_name,
                2,
                &ReadConfig {
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
        let group_id = unique_id();
        let read_config = ReadConfig {
            max_record_num: 1,
            ..Default::default()
        };

        // read m1
        let offset = 0;
        let res = mysql_adapter
            .read_by_offset(&shard.shard_name, offset, &read_config)
            .await
            .unwrap();

        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data.to_vec()).unwrap(),
            ms1
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().metadata.offset,
        );

        mysql_adapter
            .commit_offset(&group_id, &offset_data)
            .await
            .unwrap();

        // read ms2
        let offset = mysql_adapter.get_offset_by_group(&group_id).await.unwrap();

        let res = mysql_adapter
            .read_by_offset(
                &shard.shard_name,
                offset.first().unwrap().offset + 1,
                &read_config,
            )
            .await
            .unwrap();

        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data.to_vec()).unwrap(),
            ms2
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().metadata.offset,
        );
        mysql_adapter
            .commit_offset(&group_id, &offset_data)
            .await
            .unwrap();

        // read m3
        let offset: Vec<crate::storage::ShardOffset> =
            mysql_adapter.get_offset_by_group(&group_id).await.unwrap();

        let res = mysql_adapter
            .read_by_offset(
                &shard.shard_name,
                offset.first().unwrap().offset + 1,
                &read_config,
            )
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data.to_vec()).unwrap(),
            ms3
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().metadata.offset,
        );
        mysql_adapter
            .commit_offset(&group_id, &offset_data)
            .await
            .unwrap();

        // read m4
        let offset = mysql_adapter.get_offset_by_group(&group_id).await.unwrap();

        let res = mysql_adapter
            .read_by_offset(
                &shard.shard_name,
                offset.first().unwrap().offset + 1,
                &read_config,
            )
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data.to_vec()).unwrap(),
            ms4
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().metadata.offset,
        );
        mysql_adapter
            .commit_offset(&group_id, &offset_data)
            .await
            .unwrap();

        // delete shard
        mysql_adapter.delete_shard(&shard.shard_name).await.unwrap();

        // check if the shard is deleted
        let shards = mysql_adapter.list_shard(&shard.shard_name).await.unwrap();

        assert_eq!(shards.len(), 0);

        mysql_adapter.close().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_concurrent_batch_write_test() {
        let mysql_adapter =
            Arc::new(MySQLStorageAdapter::new(StorageDriverMySQLConfig::default()).unwrap());

        let shard_name = "test-concurrent".to_string();

        // step 1: create shard
        let shard = ShardInfo {
            shard_name: shard_name.clone(),
            replica_num: 1,
            ..Default::default()
        };
        mysql_adapter.create_shard(&shard).await.unwrap();

        let mut handles = Vec::new();

        // spawn 100 tasks to write data concurrently
        for tid in 0..100 {
            let mysql_adapter = mysql_adapter.clone();
            let shard = shard.clone();

            let handle = tokio::spawn(async move {
                let mut data = Vec::new();
                let header = vec![Header {
                    name: "n1".to_string(),
                    value: "v1".to_string(),
                }];

                // push 100 records for each task
                for i in 0..100 {
                    let value = format!("test-{tid}-{i}").as_bytes().to_vec();
                    data.push(Record {
                        data: value.clone().into(),
                        key: Some(format!("k-{tid}-{i}")),
                        header: Some(header.clone()),
                        offset: None,
                        timestamp: 1737600096,
                        tags: None,
                        crc_num: calc_crc32(&value),
                    });
                }

                let offsets = mysql_adapter
                    .batch_write(&shard.shard_name, &data)
                    .await
                    .unwrap();

                assert_eq!(offsets.len(), 100);

                // read the records back
                for (idx, offset) in offsets.into_iter().enumerate() {
                    let record = mysql_adapter
                        .read_by_offset(
                            &shard.shard_name,
                            offset,
                            &ReadConfig {
                                max_record_num: 1,
                                max_size: 1024,
                            },
                        )
                        .await
                        .unwrap()
                        .first()
                        .cloned()
                        .unwrap();

                    assert_eq!(record.data, format!("test-{tid}-{idx}").as_bytes());
                    assert_eq!(record.key, Some(format!("k-{tid}-{idx}")));
                    assert_eq!(record.offset, Some(offset));
                }
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        future::join_all(handles).await;
    }
}
