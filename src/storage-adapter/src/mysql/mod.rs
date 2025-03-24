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
use common_base::{error::common::CommonError, utils::crc::calc_crc32};
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use mysql::{params, prelude::Queryable, Pool, Row};

use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};

pub struct MySQLStorageAdapter {
    pool: Pool,
}

impl MySQLStorageAdapter {
    pub fn new(pool: Pool) -> Result<Self, CommonError> {
        // init tags and groups table
        let mut conn = pool.get_conn()?;

        let create_tags_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `namespace` varchar(255) NOT NULL,
                `shard` varchar(255) NOT NULL,
                `m_offset` bigint unsigned NOT NULL,
                `tag` varchar(255) NOT NULL,
                PRIMARY KEY (`m_offset`, `tag`),
                INDEX `ns_shard_tag_offset_idx` (`namespace`, `shard`, `tag`, `m_offset`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::tags_table_name()
        );

        conn.query_drop(create_tags_table_sql)?;

        let create_groups_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `group` varchar(255) NOT NULL,
                `namespace` varchar(255) NOT NULL,
                `shard` varchar(255) NOT NULL,
                `offset` bigint unsigned NOT NULL,
                PRIMARY KEY (`group`, `namespace`, `shard`),
                INDEX `group` (`group`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::groups_table_name()
        );

        conn.query_drop(create_groups_table_sql)?;

        let create_shard_info_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `namespace` varchar(255) NOT NULL,
                `shard` varchar(255) NOT NULL,
                `info` blob,
                PRIMARY KEY (`namespace`, `shard`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::shard_info_table_name()
        );

        conn.query_drop(create_shard_info_table_sql)?;

        Ok(MySQLStorageAdapter { pool })
    }

    #[inline(always)]
    pub fn record_table_name(namespace: impl AsRef<str>, shard_name: impl AsRef<str>) -> String {
        format!("record_{}_{}", namespace.as_ref(), shard_name.as_ref())
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
    pub fn increment_id_table_name(
        namespace: impl AsRef<str>,
        shard_name: impl AsRef<str>,
    ) -> String {
        format!(
            "increment_id_{}_{}",
            namespace.as_ref(),
            shard_name.as_ref()
        )
    }
}

impl MySQLStorageAdapter {
    async fn handle_write_request(
        &self,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let mut conn = self.pool.get_conn()?;

        let mut offsets = Vec::new();

        for message in messages {
            // STEP 1: insert an empty record in the increment_id table to get an offset
            conn.query_drop(format!(
                "INSERT INTO `{}` () VALUES ();",
                Self::increment_id_table_name(&namespace, &shard_name)
            ))?;

            let offset: u64 =
                conn.query_first("SELECT LAST_INSERT_ID();")?
                    .ok_or(CommonError::CommonError(
                        "Failed to get last insert ID".to_string(),
                    ))?;

            let insert_record_sql = format!(
                "INSERT INTO `{}` (`offset`, `key`, `data`, `header`, `tags`, `ts`) VALUES (:offset, :key, :data, :header, :tags, :ts);",
                Self::record_table_name(&namespace, &shard_name)
            );

            conn.exec_drop(
                insert_record_sql,
                params! {
                    "offset" => offset - 1,   // offset is 1-based in the mysql AUTO_INCREMENT column
                    "key" => message.key,
                    "data" => message.data,
                    "header" => serde_json::to_vec(&message.header).unwrap(),
                    "tags" => serde_json::to_vec(&message.tags).unwrap(),
                    "ts" => message.timestamp,
                },
            )?;

            offsets.push(offset - 1);

            // Then we need to insert into tags table
            let insert_tags_sql = format!(
                "INSERT INTO `{}` (`namespace`, `shard`, `m_offset`, `tag`) VALUES (:namespace, :shard, :m_offset, :tag);",
                Self::tags_table_name()
            );

            conn.exec_batch(
                insert_tags_sql.as_str(),
                message.tags.into_iter().map(|tag| {
                    params! {
                        "namespace" => namespace.clone(),
                        "shard" => shard_name.clone(),
                        "m_offset" => offset - 1, // offset is 1-based in the mysql AUTO_INCREMENT column
                        "tag" => tag,
                    }
                }),
            )?;
        }

        Ok(offsets)
    }
}

#[async_trait]
impl StorageAdapter for MySQLStorageAdapter {
    async fn create_shard(&self, shard: ShardInfo) -> Result<(), CommonError> {
        let mut conn = self.pool.get_conn()?;

        let table_name = Self::record_table_name(&shard.namespace, &shard.shard_name);

        let check_table_exists_sql = format!("SHOW TABLES LIKE '{}';", table_name);

        if conn
            .query_first::<Row, _>(check_table_exists_sql)?
            .is_some()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} under namespace {} already exists",
                &shard.shard_name, &shard.namespace
            )));
        };

        let create_table_sql = format!(
            "CREATE TABLE `{}` (
                `offset` bigint unsigned PRIMARY KEY,
                `key` varchar(255) DEFAULT NULL,
                `data` blob,
                `header` blob,
                `tags` blob,
                `ts` bigint unsigned NOT NULL,
                INDEX `key_idx` (`key`),
                INDEX `ts_idx` (`ts`),
                INDEX `ts_offset_idx` (`ts`, `offset`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            table_name
        );

        conn.query_drop(create_table_sql)?;

        let insert_shard_info_sql = format!(
            "REPLACE INTO `{}` (`namespace`, `shard`, `info`) VALUES (:namespace, :shard, :info)",
            Self::shard_info_table_name()
        );

        conn.exec_drop(
            insert_shard_info_sql,
            params! {
                "namespace" => shard.namespace.clone(),
                "shard" => shard.shard_name.clone(),
                "info" => serde_json::to_vec(&shard).unwrap(),
            },
        )?;

        // create a dummy sql table with only one auto increment column
        let create_increment_id_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                `offset` bigint unsigned PRIMARY KEY AUTO_INCREMENT
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
            Self::increment_id_table_name(shard.namespace, shard.shard_name)
        );

        conn.query_drop(create_increment_id_table_sql)?;

        Ok(())
    }

    async fn list_shard(
        &self,
        namespace: String,
        shard_name: String,
    ) -> Result<Vec<ShardInfo>, CommonError> {
        let mut conn = self.pool.get_conn()?;

        if namespace.is_empty() {
            let sql = format!("SELECT info FROM `{}`;", Self::shard_info_table_name());
            let res = conn.query_map(sql, |info: Vec<u8>| {
                serde_json::from_slice::<ShardInfo>(&info).unwrap()
            })?;

            return Ok(res);
        }

        if shard_name.is_empty() {
            let sql = format!(
                "SELECT info FROM `{}` WHERE namespace = :namespace;",
                Self::shard_info_table_name()
            );

            let res = conn.exec_map(
                sql,
                params! {
                    "namespace" => namespace,
                },
                |info: Vec<u8>| serde_json::from_slice::<ShardInfo>(&info).unwrap(),
            )?;

            return Ok(res);
        }

        let sql = format!(
            "SELECT info FROM `{}` WHERE namespace = :namespace AND shard = :shard;",
            Self::shard_info_table_name()
        );

        let res = conn.exec_map(
            sql,
            params! {
                "namespace" => namespace,
                "shard" => shard_name,
            },
            |info: Vec<u8>| serde_json::from_slice::<ShardInfo>(&info).unwrap(),
        )?;

        Ok(res)
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        let table_name = Self::record_table_name(&namespace, &shard_name);

        let check_table_exists_sql = format!("SHOW TABLES LIKE '{}';", table_name);

        let mut conn = self.pool.get_conn()?;

        if conn
            .query_first::<Row, _>(check_table_exists_sql)?
            .is_none()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} under namespace {} does not exist",
                &shard_name, &namespace
            )));
        };

        let drop_table_sql = format!("DROP TABLE IF EXISTS `{}`", table_name);

        conn.query_drop(drop_table_sql)?;

        let drop_shard_info_sql = format!(
            "DELETE FROM `{}` WHERE namespace = :namespace AND shard = :shard",
            Self::shard_info_table_name()
        );

        conn.exec_drop(
            drop_shard_info_sql,
            params! {
                "namespace" => namespace,
                "shard" => shard_name,
            },
        )?;

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

        Ok(offsets.first().cloned().ok_or(CommonError::CommonError(
            "Failed to get offset. The vector is empty!".to_string(),
        ))?)
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
        let mut conn = self.pool.get_conn()?;

        let sql = format!(
            "SELECT `offset`, `key`, `data`, `header`, `tags`, `ts`
            FROM `{}`
            WHERE `offset` >= :offset
            ORDER BY `offset`
            LIMIT :limit;",
            Self::record_table_name(&namespace, &shard_name)
        );

        let res: Vec<Record> = conn.exec_map(
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
                Record {
                    offset: Some(offset), // offset is 1-based in the database
                    key,
                    data: data.clone(),
                    header: serde_json::from_slice(&header).unwrap(),
                    tags: serde_json::from_slice(&tags).unwrap(),
                    timestamp: ts,
                    delay_timestamp: 0,
                    crc_num: calc_crc32(&data),
                }
            },
        )?;

        Ok(res)
    }

    async fn read_by_tag(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        tag: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let mut conn = self.pool.get_conn()?;

        let sql = format!(
            "SELECT `r.offset`,`r.key`,`r.data`,`r.header`,`r.tags`,`r.ts`
            FROM
                `{}` l LEFT JOIN `{}` r on l.m_offset = r.offset
            WHERE l.tag = :tag and l.m_offset >= :offset and l.namespace = :namespace and l.shard = :shard
            ORDER BY l.m_offset
            LIMIT :limit",
            Self::tags_table_name(),
            Self::record_table_name(&namespace, &shard_name)
        );

        let res: Vec<Record> = conn.exec_map(
            sql,
            params! {
                "tag" => tag,
                "offset" => offset,
                "namespace" => namespace,
                "shard" => shard_name,
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
                Record {
                    offset: Some(offset), // offset is 1-based in the database
                    key,
                    data: data.clone(),
                    header: serde_json::from_slice(&header).unwrap(),
                    tags: serde_json::from_slice(&tags).unwrap(),
                    timestamp: ts,
                    delay_timestamp: 0,
                    crc_num: calc_crc32(&data),
                }
            },
        )?;

        Ok(res)
    }

    async fn read_by_key(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        key: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let mut conn = self.pool.get_conn()?;

        let sql = format!(
            "SELECT `offset`, `key`, `data`, `header`, `tags`, `ts`
            FROM {}
            WHERE `offset` >= :offset AND `key` = :key
            ORDER BY `offset`
            LIMIT :limit",
            Self::record_table_name(&namespace, &shard_name)
        );

        let res = conn
            .exec_first(
                sql,
                params! {
                    "offset" => offset,
                    "key" => key,
                    "limit" => read_config.max_record_num,
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
                )| Record {
                    offset: Some(offset),
                    key,
                    data: data.clone(),
                    header: serde_json::from_slice(&header).unwrap(),
                    tags: serde_json::from_slice(&tags).unwrap(),
                    timestamp: ts,
                    delay_timestamp: 0,
                    crc_num: calc_crc32(&data),
                },
            )
            .ok_or(CommonError::CommonError("No record found".to_string()))?;

        Ok(vec![res])
    }

    async fn get_offset_by_timestamp(
        &self,
        namespace: String,
        shard_name: String,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let mut conn = self.pool.get_conn()?;

        let sql = format!(
            "SELECT `offset`
            FROM `{}`
            WHERE `ts` >= :ts
            ORDER BY `ts`
            LIMIT 1",
            Self::record_table_name(&namespace, &shard_name)
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
        .map_err(|e| CommonError::CommonError(format!("Failed to get offset by timestamp: {}", e)))
    }

    async fn get_offset_by_group(
        &self,
        group_name: String,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        let mut conn = self.pool.get_conn()?;

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
        group_name: String,
        namespace: String,
        offset: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let mut conn = self.pool.get_conn()?;

        let sql = format!(
            "REPLACE INTO `{}` (`group`, `namespace`, `shard`, `offset`) VALUES (:group, :namespace, :shard, :offset)",
            Self::groups_table_name()
        );

        conn.exec_batch(
            sql,
            offset.into_iter().map(|(shard, offset)| {
                params! {
                    "group" => group_name.clone(),
                    "namespace" => namespace.clone(),
                    "shard" => shard,
                    "offset" => offset,
                }
            }),
        )?;

        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use common_base::{tools::unique_id, utils::crc::calc_crc32};
    use futures::future;
    use metadata_struct::adapter::{
        read_config::ReadConfig,
        record::{Header, Record},
    };

    use mysql::{prelude::Queryable, Pool};
    use third_driver::mysql::build_mysql_conn_pool;

    use super::MySQLStorageAdapter;
    use crate::storage::{ShardInfo, StorageAdapter};

    async fn clean_resources(pool: Pool) {
        let mut conn = pool.get_conn().unwrap();

        conn.query::<String, _>("SHOW TABLES;")
            .unwrap()
            .into_iter()
            .for_each(|row| {
                conn.query_drop(format!("DROP TABLE IF EXISTS `{}`", row))
                    .unwrap();
            });
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_create_shard() {
        let addr = "mysql://root@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();

        let mysql_adapter = MySQLStorageAdapter::new(pool.clone()).unwrap();
        let shard_name = String::from("test");
        let namespace = unique_id();
        mysql_adapter
            .create_shard(ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        mysql_adapter
            .delete_shard(namespace, shard_name)
            .await
            .unwrap();

        clean_resources(pool).await;
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_batch_write() {
        let addr = "mysql://root@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool.clone()).unwrap();
        let shard_name = String::from("test");
        let namespace = unique_id();
        mysql_adapter
            .create_shard(ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();
        let mut data = Vec::new();
        let header = vec![Header {
            name: "n1".to_string(),
            value: "v1".to_string(),
        }];

        let value = "test1".to_string().as_bytes().to_vec();
        data.push(Record {
            data: value.clone(),
            key: "k1".to_string(),
            header: header.clone(),
            offset: None,
            timestamp: 1737600096,
            tags: vec![],
            delay_timestamp: 0,
            crc_num: calc_crc32(&value),
        });

        let value = "test2".to_string().as_bytes().to_vec();
        data.push(Record {
            data: value.clone(),
            key: "k2".to_string(),
            header: header.clone(),
            offset: None,
            timestamp: 1737600097,
            tags: vec![],
            delay_timestamp: 0,
            crc_num: calc_crc32(&value),
        });

        let result = mysql_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 1);

        let records = mysql_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                0,
                ReadConfig {
                    max_record_num: 10,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].data, "test1".to_string().as_bytes().to_vec());
        assert_eq!(records[1].data, "test2".to_string().as_bytes().to_vec());

        assert_eq!(records[0].key, "k1");
        assert_eq!(records[1].key, "k2");

        assert_eq!(records[0].offset, Some(0));
        assert_eq!(records[1].offset, Some(1));

        clean_resources(pool).await;
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_stream_read_write() {
        let addr = "mysql://root@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();

        let mysql_adapter = MySQLStorageAdapter::new(pool.clone()).unwrap();

        let namespace = unique_id();
        let shard_name = "test-11".to_string();

        // step 1: create shard
        mysql_adapter
            .create_shard(ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let shards = mysql_adapter
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

        let result = mysql_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);

        // read previous records
        assert_eq!(
            mysql_adapter
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

        let result = mysql_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();

        // read from offset 2
        let result_read = mysql_adapter
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
        let group_id = unique_id();
        let read_config = ReadConfig {
            max_record_num: 1,
            ..Default::default()
        };

        // read m1
        let offset = 0;
        let res = mysql_adapter
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

        mysql_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read ms2
        let offset = mysql_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = mysql_adapter
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
        mysql_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read m3
        let offset: Vec<crate::storage::ShardOffset> = mysql_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = mysql_adapter
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
        mysql_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read m4
        let offset = mysql_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = mysql_adapter
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
        mysql_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // delete shard
        mysql_adapter
            .delete_shard(namespace.clone(), shard_name.clone())
            .await
            .unwrap();

        // check if the shard is deleted
        let shards = mysql_adapter
            .list_shard(namespace, shard_name)
            .await
            .unwrap();

        assert_eq!(shards.len(), 0);

        mysql_adapter.close().await.unwrap();

        clean_resources(pool).await;
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_concurrent_batch_write_test() {
        let addr = "mysql://root@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();

        let mysql_adapter = Arc::new(MySQLStorageAdapter::new(pool.clone()).unwrap());

        let namespace = unique_id();
        let shard_name = "test-concurrent".to_string();

        // step 1: create shard
        mysql_adapter
            .create_shard(ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let mut handles = Vec::new();

        // spawn 10 tasks to write data concurrently
        for tid in 0..10 {
            let mysql_adapter = mysql_adapter.clone();
            let namespace = namespace.clone();
            let shard_name = shard_name.clone();

            let handle = tokio::spawn(async move {
                let mut data = Vec::new();
                let header = vec![Header {
                    name: "n1".to_string(),
                    value: "v1".to_string(),
                }];

                // push 10 records for each task
                for i in 0..10 {
                    let value = format!("test-{}-{}", tid, i).as_bytes().to_vec();
                    data.push(Record {
                        data: value.clone(),
                        key: format!("k-{}-{}", tid, i),
                        header: header.clone(),
                        offset: None,
                        timestamp: 1737600096,
                        tags: vec![],
                        delay_timestamp: 0,
                        crc_num: calc_crc32(&value),
                    });
                }

                let offsets = mysql_adapter
                    .batch_write(namespace.clone(), shard_name.clone(), data)
                    .await
                    .unwrap();

                assert_eq!(offsets.len(), 10);

                // read the records back
                for (idx, offset) in offsets.into_iter().enumerate() {
                    let record = mysql_adapter
                        .read_by_offset(
                            namespace.clone(),
                            shard_name.clone(),
                            offset,
                            ReadConfig {
                                max_record_num: 1,
                                max_size: 1024,
                            },
                        )
                        .await
                        .unwrap()
                        .first()
                        .cloned()
                        .unwrap();

                    assert_eq!(record.data, format!("test-{}-{}", tid, idx).as_bytes());
                    assert_eq!(record.key, format!("k-{}-{}", tid, idx));
                    assert_eq!(record.offset, Some(offset));
                }
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        future::join_all(handles).await;

        clean_resources(pool).await;
    }
}
